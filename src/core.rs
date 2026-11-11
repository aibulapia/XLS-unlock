// 엑셀 Unlock - 해제 핵심 로직 (OS 무관)
//
// xlsx/xlsm 는 내부가 ZIP 이다. 그 안의 XML 에서 보호 태그만 지우고
// 나머지는 손대지 않은 채 새 ZIP 으로 다시 묶는다.

use regex::bytes::Regex;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};

/// 결과 폴더 이름
pub const OUT_DIR_NAME: &str = "엑셀 Unlock";
/// 결과 파일에 붙는 꼬리말
pub const SUFFIX: &str = "_해제";

/// 한 파일 처리 결과
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// 보호를 n개 지웠다
    Unlocked { out: PathBuf, removed: usize },
    /// 걸린 보호가 없었다 (사본만 생성)
    NoProtection { out: PathBuf },
    /// 파일 열기 암호가 걸려 있다 (ZIP 이 아님)
    Encrypted,
    /// 대상이 아니다
    Skipped(String),
    /// 실패
    Failed(String),
}

/// 저장 위치 방식
#[derive(Debug, Clone, PartialEq)]
pub enum SaveMode {
    /// 원본 파일이 있는 폴더 밑에 OUT_DIR_NAME 폴더를 만든다
    BesideSource,
    /// 지정한 폴더 한 곳에만 모은다
    Fixed(PathBuf),
}

impl Default for SaveMode {
    fn default() -> Self {
        SaveMode::BesideSource
    }
}

/// 지워야 할 보호 태그들.
/// Regex 컴파일은 비싸므로 한 번 만들어 재사용한다.
pub struct Patterns {
    list: Vec<Regex>,
}

impl Patterns {
    pub fn new() -> Self {
        // (?s) = . 이 줄바꿈도 포함하도록
        let raw = [
            r"<sheetProtection\b[^>]*?/>",            // 시트 보호
            r"<workbookProtection\b[^>]*?/>",         // 통합문서 구조/창 보호
            r"<fileSharing\b[^>]*?/>",                // 읽기전용 권장 암호
            r"(?s)<protectedRanges>.*?</protectedRanges>", // 범위 편집 허용
            r"(?s)<protectedRange\b[^>]*?/>",
        ];
        Patterns {
            list: raw
                .iter()
                .map(|p| Regex::new(p).expect("내장 패턴은 항상 올바르다"))
                .collect(),
        }
    }

    /// data 에서 보호 태그를 지우고, 실제로 잠금 역할을 하던 개수를 함께 돌려준다.
    ///
    /// `<workbookProtection/>` 처럼 속성이 하나도 없는 빈 태그는 아무것도 잠그지 않는다.
    /// (일부 프로그램이 보호를 걸지 않아도 이 태그를 써 넣는다.)
    /// 그런 태그는 지우기는 하되 "보호 n개 해제" 숫자에는 넣지 않는다.
    fn strip(&self, data: &[u8]) -> (Vec<u8>, usize) {
        let mut cur = data.to_vec();
        let mut removed = 0usize;

        for re in &self.list {
            // 먼저 지울 구간을 모은다
            let mut spans: Vec<(usize, usize)> = Vec::new();
            for m in re.find_iter(&cur) {
                if has_attribute(m.as_bytes()) {
                    removed += 1;
                }
                spans.push((m.start(), m.end()));
            }
            if spans.is_empty() {
                continue;
            }
            // 뒤에서부터 지워야 앞쪽 위치가 밀리지 않는다
            let mut next = Vec::with_capacity(cur.len());
            let mut cursor = 0usize;
            for (s, e) in &spans {
                next.extend_from_slice(&cur[cursor..*s]);
                cursor = *e;
            }
            next.extend_from_slice(&cur[cursor..]);
            cur = next;
        }

        (cur, removed)
    }
}

impl Default for Patterns {
    fn default() -> Self {
        Self::new()
    }
}

/// 태그 안에 속성이 하나라도 있는가.
/// `<sheetProtection sheet="1"/>` -> true, `<workbookProtection/>` -> false.
/// 닫는 태그(`</protectedRanges>`)까지 포함한 덩어리는 항상 실제 내용으로 본다.
fn has_attribute(tag: &[u8]) -> bool {
    // 태그 이름이 끝나는 곳(공백)을 찾고, 그 뒤에 '=' 가 있으면 속성이 있다
    let body_start = match tag.iter().position(|c| *c == b' ' || *c == b'\t' || *c == b'\n') {
        Some(i) => i,
        None => return false, // 공백이 없으면 속성도 없다
    };
    tag[body_start..].contains(&b'=')
}

/// 이 ZIP 항목을 건드려야 하는가
fn is_target(name: &str) -> bool {
    if name == "xl/workbook.xml" {
        return true;
    }
    (name.starts_with("xl/worksheets/") || name.starts_with("xl/chartsheets/"))
        && name.ends_with(".xml")
}

/// 파일 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// .xlsx / .xlsm — 속이 ZIP
    Zip,
    /// .xls — 속이 OLE 복합 문서 (BIFF8)
    Ole,
}

/// 확장자로 종류를 가린다 (대소문자 무시). 대상이 아니면 None.
pub fn kind_of(path: &Path) -> Option<Kind> {
    let e = path.extension().and_then(|e| e.to_str())?.to_ascii_lowercase();
    match e.as_str() {
        "xlsx" | "xlsm" => Some(Kind::Zip),
        "xls" => Some(Kind::Ole),
        _ => None,
    }
}

/// 확장자가 대상인가 (대소문자 무시)
pub fn is_excel_ext(path: &Path) -> bool {
    kind_of(path).is_some()
}

/// 비어 있는 이름을 찾는다.
/// 목록.xlsx -> 목록_해제.xlsx -> 목록_해제(2).xlsx -> (3) ...
pub fn next_free_name(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{stem}{SUFFIX}.{ext}"));
    if !first.exists() {
        return first;
    }
    // 번호는 넉넉히 돌리되 무한루프는 막는다
    for i in 2..10_000u32 {
        let cand = dir.join(format!("{stem}{SUFFIX}({i}).{ext}"));
        if !cand.exists() {
            return cand;
        }
    }
    // 여기까지 올 일은 사실상 없지만, 덮어쓰지 않도록 시각을 붙인다
    let uniq = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    dir.join(format!("{stem}{SUFFIX}_{uniq}.{ext}"))
}

/// 결과를 저장할 폴더를 정하고, 없으면 만든다
pub fn resolve_out_dir(src: &Path, mode: &SaveMode) -> std::io::Result<PathBuf> {
    let dir = match mode {
        SaveMode::Fixed(p) => p.clone(),
        SaveMode::BesideSource => {
            let parent = src.parent().unwrap_or_else(|| Path::new("."));
            parent.join(OUT_DIR_NAME)
        }
    };
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// ZIP 을 다시 쓰면서 보호 태그를 지운다. 지운 개수를 돌려준다.
fn rewrite<R: Read + Seek, W: Write + Seek>(
    reader: R,
    writer: W,
    pat: &Patterns,
) -> Result<usize, zip::result::ZipError> {
    let mut zin = zip::ZipArchive::new(reader)?;
    let mut zout = zip::ZipWriter::new(writer);
    let mut removed_total = 0usize;

    for i in 0..zin.len() {
        let mut item = zin.by_index(i)?;
        let name = item.name().to_string();

        if is_target(&name) {
            let mut buf = Vec::with_capacity(item.size() as usize);
            item.read_to_end(&mut buf).map_err(zip::result::ZipError::Io)?;
            let (out, n) = pat.strip(&buf);
            removed_total += n;

            let opts: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            zout.start_file(&name, opts)?;
            zout.write_all(&out).map_err(zip::result::ZipError::Io)?;
        } else {
            // 건드리지 않는 항목은 압축을 풀지 않고 그대로 복사한다.
            // 원본 바이트가 그대로 보존되고 속도도 훨씬 빠르다.
            zout.raw_copy_file(item)?;
        }
    }

    zout.finish()?;
    Ok(removed_total)
}

/// 파일 하나를 처리한다
pub fn unlock_one(src: &Path, mode: &SaveMode, pat: &Patterns) -> Outcome {
    if src.is_dir() {
        return Outcome::Skipped(format!(
            "{} — 폴더는 건너뜁니다",
            file_name_of(src)
        ));
    }
    if !src.exists() {
        return Outcome::Failed(format!("{} — 파일을 찾을 수 없습니다", file_name_of(src)));
    }
    let Some(kind) = kind_of(src) else {
        return Outcome::Skipped(format!(
            "{} — xlsx/xlsm/xls 파일이 아닙니다",
            file_name_of(src)
        ));
    };

    // 이미 해제된 결과물을 다시 넣는 실수를 막는다
    if let Some(stem) = src.file_stem().and_then(|s| s.to_str()) {
        if stem.contains(SUFFIX) {
            return Outcome::Skipped(format!(
                "{} — 이미 해제된 파일로 보입니다",
                file_name_of(src)
            ));
        }
    }

    // 속을 확인한다. 확장자만 믿으면 안 된다.
    //  - xlsx/xlsm 인데 ZIP 이 아니면 -> 열기 암호가 걸린 것
    //  - xls 인데 OLE 가 아니면 -> 웹에서 내려받은 HTML 을 .xls 로 저장한 것
    match kind {
        Kind::Zip => match is_zip(src) {
            Ok(false) => return Outcome::Encrypted,
            Err(e) => {
                return Outcome::Failed(format!("{} — 열 수 없습니다: {e}", file_name_of(src)))
            }
            Ok(true) => {}
        },
        Kind::Ole => match crate::xls::is_ole(src) {
            Ok(false) => {
                return Outcome::Skipped(format!(
                    "{} — 이름만 .xls 이고 실제로는 엑셀 파일이 아닙니다 (보호 없음)",
                    file_name_of(src)
                ))
            }
            Err(e) => {
                return Outcome::Failed(format!("{} — 열 수 없습니다: {e}", file_name_of(src)))
            }
            Ok(true) => {}
        },
    }

    let out_dir = match resolve_out_dir(src, mode) {
        Ok(d) => d,
        Err(e) => {
            return Outcome::Failed(format!(
                "{} — 저장 폴더를 만들지 못했습니다: {e}",
                file_name_of(src)
            ))
        }
    };

    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("파일")
        .to_string();
    let ext = src
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("xlsx")
        .to_string();
    let out = next_free_name(&out_dir, &stem, &ext);

    match kind {
        Kind::Zip => unlock_zip(src, &out, pat),
        Kind::Ole => unlock_ole(src, &out),
    }
}

/// xlsx / xlsm : ZIP 을 다시 묶으면서 보호 태그를 지운다
fn unlock_zip(src: &Path, out: &Path, pat: &Patterns) -> Outcome {
    let fin = match File::open(src) {
        Ok(f) => f,
        Err(e) => return Outcome::Failed(format!("{} — 열 수 없습니다: {e}", file_name_of(src))),
    };
    let fout = match File::create(out) {
        Ok(f) => f,
        Err(e) => {
            return Outcome::Failed(format!("{} — 저장하지 못했습니다: {e}", file_name_of(src)))
        }
    };

    match rewrite(BufReader::new(fin), BufWriter::new(fout), pat) {
        Ok(0) => Outcome::NoProtection { out: out.to_path_buf() },
        Ok(n) => Outcome::Unlocked { out: out.to_path_buf(), removed: n },
        Err(e) => {
            // 실패하면 반쯤 쓰다 만 파일을 남기지 않는다
            let _ = fs::remove_file(out);
            match e {
                zip::result::ZipError::InvalidArchive(_) => Outcome::Encrypted,
                other => Outcome::Failed(format!("{} — 실패: {other}", file_name_of(src))),
            }
        }
    }
}

/// xls : 복사해 두고 그 안에서 보호 값만 0 으로 덮는다
fn unlock_ole(src: &Path, out: &Path) -> Outcome {
    if let Err(e) = fs::copy(src, out) {
        return Outcome::Failed(format!("{} — 복사하지 못했습니다: {e}", file_name_of(src)));
    }

    match crate::xls::unlock_in_place(out) {
        crate::xls::XlsResult::Unlocked(n) => {
            Outcome::Unlocked { out: out.to_path_buf(), removed: n }
        }
        crate::xls::XlsResult::NoProtection => {
            Outcome::NoProtection { out: out.to_path_buf() }
        }
        crate::xls::XlsResult::Encrypted => {
            let _ = fs::remove_file(out);
            Outcome::Encrypted
        }
        crate::xls::XlsResult::Failed(m) => {
            let _ = fs::remove_file(out);
            Outcome::Failed(format!("{} — {m}", file_name_of(src)))
        }
    }
}

/// ZIP 인지 헤더로 확인한다 (PK\x03\x04)
fn is_zip(path: &Path) -> std::io::Result<bool> {
    let mut f = File::open(path)?;
    let mut head = [0u8; 4];
    match f.read(&mut head)? {
        4 => Ok(&head == b"PK\x03\x04"),
        _ => Ok(false),
    }
}

pub fn file_name_of(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("(이름 없음)")
        .to_string()
}
