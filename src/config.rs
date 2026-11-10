// 저장 경로 설정을 파일에 보관한다.
// 위치: %APPDATA%\엑셀Unlock\settings.txt
// 형식: 아주 단순한 한 줄짜리. 라이브러리를 더 끌어오지 않기 위함이다.
//   mode=beside
//   mode=fixed
//   path=D:\작업\엑셀 Unlock

use crate::core::SaveMode;
use std::fs;
use std::path::{Path, PathBuf};

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("엑셀Unlock"))
}

fn config_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.txt"))
}

/// 설정을 읽는다. 없거나 깨졌으면 기본값(원본 폴더 옆)을 쓴다.
pub fn load() -> SaveMode {
    let Some(f) = config_file() else {
        return SaveMode::BesideSource;
    };
    let Ok(text) = fs::read_to_string(&f) else {
        return SaveMode::BesideSource;
    };

    let mut mode_fixed = false;
    let mut path: Option<PathBuf> = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("mode=") {
            mode_fixed = v.trim() == "fixed";
        } else if let Some(v) = line.strip_prefix("path=") {
            let v = v.trim();
            if !v.is_empty() {
                path = Some(PathBuf::from(v));
            }
        }
    }

    match (mode_fixed, path) {
        // 저장해 둔 폴더가 사라졌으면 조용히 기본값으로 돌아간다.
        // 엉뚱한 곳에 쓰는 것보다 낫다.
        (true, Some(p)) if p.is_dir() => SaveMode::Fixed(p),
        _ => SaveMode::BesideSource,
    }
}

/// 설정을 저장한다. 실패해도 프로그램은 계속 돌아간다.
pub fn save(mode: &SaveMode) -> std::io::Result<()> {
    let Some(dir) = config_dir() else {
        return Ok(());
    };
    fs::create_dir_all(&dir)?;
    let Some(f) = config_file() else {
        return Ok(());
    };

    let body = match mode {
        SaveMode::BesideSource => "mode=beside\n".to_string(),
        SaveMode::Fixed(p) => format!("mode=fixed\npath={}\n", p.display()),
    };
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

/// 마지막으로 결과가 저장된 폴더를 기억해 두면 '결과 폴더 열기'에 쓸 수 있다
pub fn last_out_dir(mode: &SaveMode, fallback: Option<&Path>) -> Option<PathBuf> {
    match mode {
        SaveMode::Fixed(p) => Some(p.clone()),
        SaveMode::BesideSource => fallback.map(|p| p.to_path_buf()),
    }
}
