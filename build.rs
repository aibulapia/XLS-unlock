// exe 에 아이콘 / 버전정보 / 매니페스트를 박아 넣는다.
//
// 두 가지 툴체인을 모두 지원한다.
//   - MSVC  (x86_64-pc-windows-msvc) : llvm-rc 또는 rc.exe  -> .res
//   - mingw (x86_64-pc-windows-gnu)  : windres              -> .o
//
// 배포본은 MSVC 로 만드는 쪽이 낫다. mingw 로 만든 exe 는
// 섹션 구성(.CRT/.tls/.xdata)이 백신 휴리스틱에 자주 걸려
// "바이러스 발견"으로 잘못 잡히는 일이 많다.
//
// 주의: 빌드 스크립트는 *빌드하는 쪽*(여기서는 리눅스)에서 돌아간다.
// 그래서 #[cfg(windows)] 로 감싸면 통째로 건너뛴다.
// 만들려는 쪽이 윈도우인지는 CARGO_CFG_TARGET_OS 로 봐야 한다.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=assets/resource.rc");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR 은 항상 있다"));
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let rc = assets.join("resource.rc");

    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let artifact = if env == "msvc" {
        build_res(&rc, &assets, &out_dir)
    } else {
        build_obj(&rc, &assets, &out_dir)
    };

    // winres 처럼 정적 라이브러리로 넘기면 링커가 "참조가 없다"며 통째로 버린다.
    // 그러면 아이콘도 매니페스트도 조용히 사라진다. 직접 링크해야 한다.
    println!("cargo:rustc-link-arg-bins={}", artifact.display());
}

/// MSVC 툴체인: .rc -> .res
fn build_res(rc: &Path, assets: &Path, out_dir: &Path) -> PathBuf {
    let out = out_dir.join("resource.res");
    let tool = first_available(&["llvm-rc", "llvm-rc-18", "rc.exe", "rc"])
        .expect("리소스 컴파일러를 찾지 못했습니다 (llvm-rc 또는 rc.exe 가 필요합니다)");

    // /C 65001 : .rc 파일이 UTF-8 이라고 알려준다. 없으면 한글이 깨진다.
    let st = Command::new(&tool)
        .current_dir(assets)
        .args(["/C", "65001", "/FO"])
        .arg(&out)
        .arg(rc)
        .status()
        .unwrap_or_else(|e| panic!("{tool} 실행 실패: {e}"));

    assert!(st.success(), "{tool} 가 리소스 컴파일에 실패했습니다");
    out
}

/// mingw 툴체인: .rc -> .o
fn build_obj(rc: &Path, assets: &Path, out_dir: &Path) -> PathBuf {
    let out = out_dir.join("resource.o");
    let tool = first_available(&["x86_64-w64-mingw32-windres", "windres"])
        .expect("windres 를 찾지 못했습니다 (mingw-w64 가 필요합니다)");

    let st = Command::new(&tool)
        .current_dir(assets)
        .args(["--codepage", "65001", "-i"])
        .arg(rc)
        .arg("-o")
        .arg(&out)
        .args(["-O", "coff"])
        .status()
        .unwrap_or_else(|e| panic!("{tool} 실행 실패: {e}"));

    assert!(st.success(), "{tool} 가 리소스 컴파일에 실패했습니다");
    out
}

/// 후보 중 실제로 실행 가능한 첫 번째를 고른다
fn first_available(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find(|n| {
            Command::new(n)
                .arg("/?")
                .output()
                .or_else(|_| Command::new(n).arg("--version").output())
                .is_ok()
        })
        .map(|s| s.to_string())
}
