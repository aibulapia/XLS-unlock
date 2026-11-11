<div align="center">

<img src="icons/엑셀Unlock_아이콘_256.png" width="128" alt="엑셀 Unlock">

# 엑셀 Unlock V. 0.7

**엑셀 시트보호를 풀어주는 무설치 윈도우 프로그램**

설치 필요 없음 · 파이썬 필요 없음 · exe 하나로 끝

[⬇ 다운로드 (1.5MB)](https://github.com/aibulapia/XLS-unlock/raw/main/release/XLS-unlock_v0.7.zip) · [51work.bid](https://51work.bid)

</div>

---

## 이게 뭔가요

회사에서 받은 엑셀 파일이 열리긴 하는데 셀을 못 고칠 때가 있습니다. **시트 보호**가 걸린 겁니다.

이 프로그램에 그 파일을 끌어다 놓으면, 보호를 푼 사본을 만들어 줍니다. **원본은 손대지 않습니다.**

## 쓰는 법

1. [`XLS-unlock_v0.7.zip`](https://github.com/aibulapia/XLS-unlock/raw/main/release/XLS-unlock_v0.7.zip) 을 받아서 압축을 풉니다
2. `엑셀 Unlock.exe` 를 아무 폴더에나 두고 실행합니다
3. 엑셀 파일을 창 안에 끌어다 놓습니다 (여러 개 한꺼번에 가능)

결과는 원본이 있던 폴더 밑에 **`엑셀 Unlock`** 폴더가 생기고 그 안에 저장됩니다.

```
D:\공사\하계공사목록.xlsx                    ← 원본, 그대로 남음
D:\공사\엑셀 Unlock\하계공사목록_해제.xlsx    ← 결과
```

파일을 넣는 방법은 세 가지입니다.

- 창 안에 끌어다 놓기
- `파일 선택` 버튼으로 고르기
- exe 아이콘 위에 파일을 끌어다 놓기

같은 이름이 이미 있으면 묻지 않고 `(2)`, `(3)` 번호를 붙입니다. 덮어쓰는 일은 없습니다.

결과를 한 곳에 모으고 싶으면 `저장 경로 설정` 에서 폴더를 지정하면 됩니다. 설정은 다음 실행에도 유지됩니다.

## ⚠ 처음 실행할 때 경고가 뜹니다

```
Windows의 PC 보호
Microsoft Defender SmartScreen에서 인식할 수 없는 앱의 시작을 차단했습니다.
앱: 엑셀 Unlock.exe
게시자: 알 수 없는 게시자
```

**바이러스가 아닙니다.** 개인이 만든 프로그램에 유료 코드 서명을 넣지 않아서 나오는 안내입니다.

> **[추가 정보]** 클릭 → **[실행]** 버튼

한 번 실행하면 다음부터는 뜨지 않습니다.

파일이 바뀌지 않았는지 직접 확인하시려면 아래 값과 비교하시면 됩니다.

```
XLS-unlock_v0.7.zip
SHA-256  4a9a9597238be6cf64401bc13a2e82a26fd3fe5525b9f43eb51cd68f2424b57c
```

<details>
<summary>왜 서명을 안 넣었나요?</summary>

코드 서명 인증서는 연 30만원 안팎입니다. 그리고 2024년부터 마이크로소프트가 EV 인증서의 즉시 통과 특례를 없애서, **돈을 내도 경고가 바로 사라지지 않습니다.** 다운로드가 쌓여 평판이 생겨야 없어집니다. 개인이 만들어 무료로 나누는 프로그램에는 수지가 맞지 않아 넣지 않았습니다.

확인하려면 [Microsoft 문서](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)를 보시면 됩니다.

</details>

## 풀 수 있는 것, 못 푸는 것

| 상황 | 가능 여부 |
|---|---|
| 시트 보호 (셀 수정 잠금) | ✅ |
| 통합 문서 보호 (시트 추가·삭제·이동 잠금) | ✅ |
| 읽기 전용 권장 암호 | ✅ |
| 범위 편집 허용 설정 | ✅ |
| 도형·시나리오 보호 | ✅ |
| **파일 열기 암호** (파일 열 때 암호를 묻는 것) | ❌ |

마지막 것만 안 됩니다. 이건 파일 전체가 실제로 암호화돼 있어서, 암호를 모르면 누구도 열 수 없습니다. 그런 파일을 넣으면 "열기 암호가 걸린 파일입니다" 라고 알려줍니다.

> **왜 시트 보호는 풀리나요?**
> 시트 보호는 암호화가 아니라 "엑셀에서 수정하지 마세요" 라는 **표시**입니다.
> 이 프로그램은 그 표시만 지우고 나머지는 그대로 둡니다.
> 수식, 서식, 차트, 매크로는 건드리지 않습니다.
>
> `.xls` 는 424,448바이트 중 **22바이트만** 바뀝니다. 파일 크기도 그대로입니다.

## 대상 파일

`.xlsx` · `.xlsm` · `.xls` · Windows 64bit

구형 `.xls` 도 됩니다. 속이 완전히 달라서(ZIP 이 아니라 OLE 복합 문서) 따로 처리합니다.

웹 시스템에서 내려받은 "이름만 `.xls`" 인 파일(실제로는 HTML)은 애초에 시트보호가 없습니다. 그런 파일은 그렇다고 알려줍니다.

---

## 만들기 (개발자용)

### 필요한 것

- Rust (stable)
- 리눅스에서 만든다면 [cargo-xwin](https://github.com/rust-cross/cargo-xwin) 과 `llvm-rc`

### 배포본 (MSVC 툴체인)

```bash
rustup target add x86_64-pc-windows-msvc
cargo install cargo-xwin                       # 리눅스에서 만들 때만
cargo xwin build --release --target x86_64-pc-windows-msvc
```

**배포본은 반드시 MSVC 로 만드십시오.** mingw(GCC) 로 만든 exe 는 섹션 구성(`.CRT`/`.xdata`/`.idata`)이 백신 휴리스틱에 걸려 "바이러스 발견" 오탐이 자주 납니다. 실제로 그렇게 당했습니다.

### mingw 툴체인 (개발용)

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

### 해제 로직만 따로 테스트

GUI 없이 콘솔에서 돌려볼 수 있습니다. 리눅스에서도 됩니다.

```bash
cargo build --bin coretest
./target/debug/coretest 파일.xlsx [--out 저장폴더]
```

## 구조

```
src/
  main.rs      진입점
  core.rs      공통 해제 로직 + xlsx/xlsm (ZIP) 처리
  xls.rs       구형 xls (OLE/BIFF8) 처리
  config.rs    저장 경로·마지막 결과 폴더 보관
  gui.rs       Win32 화면
  bin/
    coretest.rs  GUI 없이 로직만 돌리는 콘솔 도구
assets/
  app.ico        exe 아이콘
  app.manifest   비주얼 스타일 + DPI 인식
  resource.rc    아이콘·버전정보·매니페스트 묶음
icons/           아이콘 원본(SVG) 및 각 크기 PNG
release/         배포용 exe (압축)
```

### 쓰인 것

| | |
|---|---|
| 언어 | Rust |
| 화면 | [native-windows-gui](https://github.com/gabdube/native-windows-gui) (Win32 직접 호출) |
| ZIP | [zip](https://github.com/zip-rs/zip2) |
| OLE | [cfb](https://github.com/mdsteele/rust-cfb) |
| 패턴 | [regex](https://github.com/rust-lang/regex) |

런타임 의존성이 없어서 exe 하나(1.5MB)로 끝납니다.

## 만들면서 걸린 것들

같은 걸 만드실 분을 위해 적어 둡니다. 전부 실제로 당한 것들입니다.

**1. 빈 보호 태그를 보호로 세면 안 됩니다.**
일부 프로그램은 보호를 걸지 않아도 `<workbookProtection/>` 를 넣습니다. 속성이 하나도 없는 이 태그는 아무것도 잠그지 않습니다. 세지 말고 조용히 지우면 됩니다.

**2. `.xls` 의 `SHEETPROTECTION` 레코드는 건드리면 안 됩니다.**
보호를 지우겠다고 이 레코드를 0 으로 덮었더니, 엑셀이 **"이 파일에 문제가 있습니다"** 라며 제한된 보기로 열었습니다. 그 안에는 값이 정해져 있어야 하는 자리(`02`, `01`)가 있어서 0 으로 만들면 규격 위반이 됩니다. 보호 여부는 `PROTECT(0x0012)` 가 정하므로 그것만 0 으로 만들면 됩니다.

**3. `build.rs` 를 `#[cfg(windows)]` 로 감싸면 안 됩니다.**
빌드 스크립트는 *빌드하는 쪽* 에서 돌아갑니다. 리눅스에서 윈도우용을 만들면 그 블록이 통째로 건너뛰어져, 아이콘도 매니페스트도 없는 exe 가 조용히 나옵니다. 빌드는 성공하므로 알아채기 어렵습니다. `CARGO_CFG_TARGET_OS` 로 판단해야 합니다.

**4. 리소스 오브젝트는 직접 링크해야 합니다.**
정적 라이브러리로 넘기면 코드에서 참조하는 심볼이 없어 링커가 통째로 버립니다. 아이콘도 매니페스트도 사라집니다.

```rust
println!("cargo:rustc-link-arg-bins={}/resource.o", out_dir);
```

**5. 테스크바 아이콘은 따로 올려야 합니다.**
nwg 의 `Window::set_icon` 은 `WM_SETICON` 을 `wParam=0`(ICON_SMALL) 으로만 보냅니다. 테스크바는 ICON_BIG 을 쓰기 때문에 윈도우 기본 아이콘이 나옵니다. `ICON_BIG` 도 직접 보내야 합니다.

**6. 글꼴 크기를 지정하지 않으면 고해상도에서 깨집니다.**
nwg 는 컨트롤 크기와 글꼴을 DPI 에 맞춰 키워 주는데, 글꼴 크기를 안 주면 시스템 기본값이 들어가 배율을 안 탑니다. 결과적으로 글자가 칸을 넘치거나 반대로 너무 작아집니다.

**7. 별도 최상위 창은 이벤트 핸들러가 따로 필요합니다.**
`full_bind_event_handler` 는 `EnumChildWindows` 로 자식만 훑습니다. 설정 창처럼 자식이 아닌 창은 메인 창에만 핸들러를 걸면 버튼이 전부 먹통이 됩니다.

**8. 건드리지 않는 ZIP 항목은 `raw_copy_file` 로.**
압축을 풀었다 다시 압축하면 느리고, 원본 바이트가 바뀝니다. 그대로 복사하는 게 빠르고 안전합니다.

## 바뀐 것

| 버전 | 내용 |
|---|---|
| 0.7 | 버튼 배치 정리, 창 너비 축소, 테스크바 아이콘 수정, 결과 폴더 열기 개선 |
| 0.6 | `.xls` 지원, 창 크기 조절, 고해상도 글자 깨짐 수정, MSVC 빌드로 전환 |
| 0.5 | 첫 공개 (`.xlsx` / `.xlsm`) |

---

## 라이선스

공개. 누구나 자유롭게 쓰시면 됩니다.

## 만든이

**KIM.H,J** · aibulapia@gmail.com · [51work.bid](https://51work.bid)
