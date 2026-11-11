// 설정을 파일에 보관한다.
// 위치: %APPDATA%\엑셀Unlock\settings.txt
// 형식: 아주 단순한 한 줄짜리. 라이브러리를 더 끌어오지 않기 위함이다.
//   mode=beside | fixed
//   path=D:\작업\엑셀 Unlock      (fixed 일 때 저장할 폴더)
//   last=D:\공사\엑셀 Unlock      (마지막으로 결과를 만든 폴더)
//
// last 를 남겨 두는 이유: '결과 폴더 열기' 가 프로그램을 껐다 켠 뒤에도
// 동작하게 하려는 것이다. 없으면 켤 때마다 "아직 결과 폴더가 없습니다" 가 뜬다.

use crate::core::SaveMode;
use std::fs;
use std::path::{Path, PathBuf};

/// 설정 한 벌
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub mode: SaveMode,
    /// 마지막으로 결과를 만든 폴더
    pub last_out: Option<PathBuf>,
}

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("엑셀Unlock"))
}

fn config_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.txt"))
}

/// 설정을 읽는다. 없거나 깨졌으면 기본값(원본 폴더 옆)을 쓴다.
pub fn load() -> Settings {
    let Some(f) = config_file() else {
        return Settings::default();
    };
    let Ok(text) = fs::read_to_string(&f) else {
        return Settings::default();
    };

    let mut mode_fixed = false;
    let mut path: Option<PathBuf> = None;
    let mut last: Option<PathBuf> = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("mode=") {
            mode_fixed = v.trim() == "fixed";
        } else if let Some(v) = line.strip_prefix("path=") {
            let v = v.trim();
            if !v.is_empty() {
                path = Some(PathBuf::from(v));
            }
        } else if let Some(v) = line.strip_prefix("last=") {
            let v = v.trim();
            if !v.is_empty() {
                last = Some(PathBuf::from(v));
            }
        }
    }

    Settings {
        mode: match (mode_fixed, path) {
            // 저장해 둔 폴더가 사라졌으면 조용히 기본값으로 돌아간다.
            // 엉뚱한 곳에 쓰는 것보다 낫다.
            (true, Some(p)) if p.is_dir() => SaveMode::Fixed(p),
            _ => SaveMode::BesideSource,
        },
        // 폴더가 지워졌을 수도 있으니 살아 있는 것만 기억한다
        last_out: last.filter(|p| p.is_dir()),
    }
}

/// 설정을 저장한다. 실패해도 프로그램은 계속 돌아간다.
pub fn save(s: &Settings) -> std::io::Result<()> {
    let Some(dir) = config_dir() else {
        return Ok(());
    };
    fs::create_dir_all(&dir)?;
    let Some(f) = config_file() else {
        return Ok(());
    };

    let mut body = match &s.mode {
        SaveMode::BesideSource => "mode=beside\n".to_string(),
        SaveMode::Fixed(p) => format!("mode=fixed\npath={}\n", p.display()),
    };
    if let Some(l) = &s.last_out {
        body.push_str(&format!("last={}\n", l.display()));
    }
    fs::write(f, body)
}

/// 화면에 보여줄 저장 위치 문구
pub fn describe(mode: &SaveMode) -> String {
    match mode {
        SaveMode::BesideSource => {
            format!("원본 파일 폴더 \\ {} \\", crate::core::OUT_DIR_NAME)
        }
        SaveMode::Fixed(p) => format!("{} \\", p.display()),
    }
}

/// '결과 폴더 열기' 를 눌렀을 때 열 폴더.
///
///  - 저장 경로를 지정해 뒀으면 그 폴더
///  - 아니면 마지막으로 결과를 만든 폴더
///
/// 둘 다 없거나 사라졌으면 None.
pub fn folder_to_open(s: &Settings) -> Option<PathBuf> {
    let candidate = match &s.mode {
        SaveMode::Fixed(p) => Some(p.as_path()),
        SaveMode::BesideSource => s.last_out.as_deref(),
    };
    candidate.filter(|p| p.is_dir()).map(Path::to_path_buf)
}
