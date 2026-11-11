// 구형 엑셀(.xls, BIFF8) 보호 해제
//
// .xls 는 OLE 복합 문서(CFB) 안에 "Workbook" 스트림이 들어 있고,
// 그 스트림은 [타입(2) 길이(2) 내용(길이)] 레코드가 죽 이어진 형태다.
// 보호는 그 중 몇 개 레코드의 값으로만 표시된다.
//
//   PROTECT(0x0012)       1 이면 시트 보호
//   PASSWORD(0x0013)      암호 해시 (0 이면 암호 없음)
//   WINDOWPROTECT(0x0019) 창 보호
//   OBJECTPROTECT(0x0063) 도형 보호
//   SCENPROTECT(0x00DD)   시나리오 보호
//
// 전부 2바이트 고정이라, 값만 0 으로 덮으면 파일 크기가 그대로다.
// 크기가 안 변하니 OLE 구조를 다시 짤 필요 없이 스트림만 되쓰면 된다.
//
// SHEETPROTECTION(0x0867) 은 일부러 건드리지 않는다.
// "보호 중일 때 무엇을 허용할지"만 담은 레코드라 PROTECT 가 0 이면 의미가 없고,
// 그 안에는 값이 정해져 있어야 하는 자리(02, 01 …)가 있다.
// 거기를 0 으로 덮으면 엑셀이 "이 파일에 문제가 있습니다" 하며
// 제한된 보기로 열어 버린다. 실제로 그렇게 당했다.
//
// FILEPASS(0x002F) 가 있으면 파일 전체가 암호화된 것이라 손댈 수 없다.

use std::fs::OpenOptions;
use std::io::{Read, Seek, Write};
use std::path::Path;

const REC_FILEPASS: u16 = 0x002F;

/// 값을 0 으로 만들면 보호가 풀리는 2바이트 레코드들
const ZERO_RECORDS: [u16; 5] = [
    0x0012, // PROTECT
    0x0013, // PASSWORD
    0x0019, // WINDOWPROTECT
    0x0063, // OBJECTPROTECT
    0x00DD, // SCENPROTECT
];

pub enum XlsResult {
    /// 보호를 n개 지웠다
    Unlocked(usize),
    /// 걸린 보호가 없었다
    NoProtection,
    /// 파일 열기 암호가 걸려 있다
    Encrypted,
    /// 처리할 수 없다
    Failed(String),
}

/// OLE 복합 문서인지 헤더로 확인한다
pub fn is_ole(path: &Path) -> std::io::Result<bool> {
    let mut f = std::fs::File::open(path)?;
    let mut head = [0u8; 8];
    match f.read(&mut head)? {
        8 => Ok(head == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
        _ => Ok(false),
    }
}

/// dst 는 이미 src 의 복사본이어야 한다. 그 자리에서 고친다.
pub fn unlock_in_place(dst: &Path) -> XlsResult {
    let file = match OpenOptions::new().read(true).write(true).open(dst) {
        Ok(f) => f,
        Err(e) => return XlsResult::Failed(format!("열 수 없습니다: {e}")),
    };

    let mut comp = match cfb::CompoundFile::open(file) {
        Ok(c) => c,
        Err(e) => return XlsResult::Failed(format!("엑셀 파일 구조를 읽지 못했습니다: {e}")),
    };

    // 스트림 이름은 보통 "Workbook", 아주 오래된 것은 "Book"
    let stream_name = ["/Workbook", "/Book"]
        .iter()
        .find(|n| comp.exists(n))
        .map(|s| s.to_string());

    let Some(name) = stream_name else {
        return XlsResult::Failed("엑셀 통합 문서가 아닙니다".into());
    };

    let mut data = Vec::new();
    {
        let mut s = match comp.open_stream(&name) {
            Ok(s) => s,
            Err(e) => return XlsResult::Failed(format!("내용을 읽지 못했습니다: {e}")),
        };
        if let Err(e) = s.read_to_end(&mut data) {
            return XlsResult::Failed(format!("내용을 읽지 못했습니다: {e}"));
        }
    }

    let original_len = data.len();

    match patch(&mut data) {
        PatchOutcome::Encrypted => return XlsResult::Encrypted,
        PatchOutcome::Changed(0) => return XlsResult::NoProtection,
        PatchOutcome::Changed(n) => {
            // 길이가 달라지면 OLE 구조가 깨진다. 절대 일어나면 안 되지만 확인한다.
            if data.len() != original_len {
                return XlsResult::Failed("내부 오류: 길이가 바뀌었습니다".into());
            }
            let write = (|| -> std::io::Result<()> {
                let mut s = comp.open_stream(&name)?;
                s.rewind()?;
                s.write_all(&data)?;
                s.flush()?;
                Ok(())
            })();
            match write {
                Ok(()) => {
                    if let Err(e) = comp.flush() {
                        return XlsResult::Failed(format!("저장하지 못했습니다: {e}"));
                    }
                    XlsResult::Unlocked(n)
                }
                Err(e) => XlsResult::Failed(format!("저장하지 못했습니다: {e}")),
            }
        }
    }
}

enum PatchOutcome {
    Encrypted,
    Changed(usize),
}

/// BIFF 레코드를 훑으며 보호 값을 0 으로 만든다
fn patch(data: &mut [u8]) -> PatchOutcome {
    let mut pos = 0usize;
    let mut changed = 0usize;

    while pos + 4 <= data.len() {
        let rec = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
        let body = pos + 4;

        // 길이가 파일을 넘어가면 더 읽지 않는다 (깨진 파일 방어)
        if body + size > data.len() {
            break;
        }

        if rec == REC_FILEPASS {
            return PatchOutcome::Encrypted;
        }

        if ZERO_RECORDS.contains(&rec) && size == 2 && (data[body] != 0 || data[body + 1] != 0) {
            data[body] = 0;
            data[body + 1] = 0;
            changed += 1;
        }

        pos = body + size;
    }

    PatchOutcome::Changed(changed)
}
