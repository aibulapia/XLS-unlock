// 해제 로직 검증용 콘솔 도구 (리눅스에서도 돌아간다)
// 사용: coretest <파일...> [--out <폴더>]

#[path = "../core.rs"]
mod core;
#[path = "../xls.rs"]
mod xls;

use core::{unlock_one, Outcome, Patterns, SaveMode};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut files: Vec<PathBuf> = Vec::new();
    let mut mode = SaveMode::BesideSource;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--out" && i + 1 < args.len() {
            mode = SaveMode::Fixed(PathBuf::from(&args[i + 1]));
            i += 2;
        } else {
            files.push(PathBuf::from(&args[i]));
            i += 1;
        }
    }

    if files.is_empty() {
        eprintln!("사용: coretest <파일...> [--out <폴더>]");
        std::process::exit(2);
    }

    let pat = Patterns::new();
    for f in &files {
        match unlock_one(f, &mode, &pat) {
            Outcome::Unlocked { out, removed } => {
                println!("OK      {} (보호 {removed}개 해제)", out.display())
            }
            Outcome::NoProtection { out } => println!("NONE    {}", out.display()),
            Outcome::Encrypted => println!("LOCKED  {} (열기 암호)", f.display()),
            Outcome::Skipped(m) => println!("SKIP    {m}"),
            Outcome::Failed(m) => println!("FAIL    {m}"),
        }
    }
}
