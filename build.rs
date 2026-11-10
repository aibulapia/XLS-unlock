// exe 에 아이콘 / 버전정보 / 매니페스트를 박아 넣는다.
//
// 주의: 빌드 스크립트는 빌드하는 쪽(여기서는 리눅스)에서 돌아간다.
// 그래서 #[cfg(windows)] 로 감싸면 안 된다. 그러면 통째로 건너뛴다.
// 만들려는 쪽이 윈도우인지는 CARGO_CFG_TARGET_OS 로 봐야 한다.

fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/app.ico");
    res.set_manifest_file("assets/app.manifest");

    res.set("FileDescription", "엑셀 Unlock - 엑셀 시트보호 해제");
    res.set("ProductName", "엑셀 Unlock");
    res.set("FileVersion", "0.5.0.0");
    res.set("ProductVersion", "0.5.0.0");
    res.set("LegalCopyright", "KIM.H,J - 51work.bid");
    res.set("OriginalFilename", "엑셀 Unlock.exe");
    res.set("CompanyName", "51work.bid");
    // 한국어(0x0412)
    res.set_language(0x0412);

    // 리눅스에서 윈도우용으로 만들 때는 mingw 쪽 도구를 써야 한다
    let host = std::env::var("HOST").unwrap_or_default();
    if !host.contains("windows") {
        res.set_windres_path("x86_64-w64-mingw32-windres");
        res.set_ar_path("x86_64-w64-mingw32-ar");
    }

    // 리소스 임베드에 실패하면 빌드를 멈춘다.
    // 조용히 넘어가면 아이콘 없는 exe 가 나와도 알아채지 못한다.
    if let Err(e) = res.compile() {
        panic!("리소스(아이콘/매니페스트) 임베드에 실패했습니다: {e}");
    }

    // winres 는 리소스를 정적 라이브러리(libresource.a)로 넘긴다.
    // 그런데 거기 들어 있는 오브젝트는 코드에서 부르는 심볼이 없어서
    // 링커가 "안 쓰네" 하고 통째로 버린다. 그러면 아이콘도 매니페스트도 사라진다.
    // 오브젝트 파일을 직접 링크해서 반드시 들어가게 한다.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR 은 항상 있다");
    let obj = std::path::Path::new(&out_dir).join("resource.o");
    if obj.exists() {
        println!("cargo:rustc-link-arg-bins={}", obj.display());
    } else {
        panic!("리소스 오브젝트를 찾지 못했습니다: {}", obj.display());
    }
}
