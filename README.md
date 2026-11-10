<div align="center">

<img src="icons/엑셀Unlock_아이콘_256.png" width="128" alt="엑셀 Unlock">

# 엑셀 Unlock V. 0.5

**엑셀 시트보호를 풀어주는 무설치 윈도우 프로그램**

설치 필요 없음 · 파이썬 필요 없음 · exe 하나로 끝

[⬇ 다운로드 (1.7MB)](https://github.com/aibulapia/XLS-unlock/raw/main/release/XLS-unlock_v0.5.zip) · [51work.bid](https://51work.bid)

</div>

---

## 이게 뭔가요

회사에서 받은 엑셀 파일이 열리긴 하는데 셀을 못 고칠 때가 있습니다. **시트 보호**가 걸린 겁니다.

이 프로그램에 그 파일을 끌어다 놓으면, 보호를 푼 사본을 만들어 줍니다. **원본은 손대지 않습니다.**

## 쓰는 법

1. [`XLS-unlock_v0.5.zip`](https://github.com/aibulapia/XLS-unlock/raw/main/release/XLS-unlock_v0.5.zip) 을 받아서 압축을 풉니다
2. `엑셀 Unlock.exe` 를 아무 폴더에나 두고 실행합니다
3. 엑셀 파일을 창 안에 끌어다 놓습니다 (여러 개 한꺼번에 가능)

결과는 원본이 있던 폴더 밑에 **`엑셀 Unlock`** 폴더가 생기고 그 안에 저장됩니다.

```
D:\공사\하계공사목록.xlsx          ← 원본, 그대로 남음
D:\공사\엑셀 Unlock\하계공사목록_해제.xlsx   ← 결과
```

파일을 넣는 방법은 세 가지입니다.

- 창 안에 끌어다 놓기
- `파일 선택` 버튼으로 고르기
- exe 아이콘 위에 파일을 끌어다 놓기

같은 이름이 이미 있으면 묻지 않고 `(2)`, `(3)` 번호를 붙입니다. 덮어쓰는 일은 없습니다.

결과를 한 곳에 모으고 싶으면 `저장 경로 설정` 에서 폴더를 지정하면 됩니다. 설정은 다음 실행에도 유지됩니다.

## 풀 수 있는 것, 못 푸는 것

| 상황 | 가능 여부 |
|---|---|
| 시트 보호 (셀 수정 잠금) | ✅ |
| 통합 문서 보호 (시트 추가·삭제·이동 잠금) | ✅ |
| 읽기 전용 권장 암호 | ✅ |
| 범위 편집 허용 설정 | ✅ |
| **파일 열기 암호** (파일 열 때 암호를 묻는 것) | ❌ |

마지막 것만 안 됩니다. 이건 파일 전체가 실제로 암호화돼 있어서, 암호를 모르면 누구도 열 수 없습니다. 그런 파일을 넣으면 "열기 암호가 걸린 파일입니다" 라고 알려줍니다.

> **왜 시트 보호는 풀리나요?**
> 시트 보호는 암호화가 아니라 "엑셀에서 수정하지 마세요" 라는 **표시**입니다.
> `.xlsx` 파일은 사실 ZIP 압축 파일이고, 그 안의 XML에 보호 표시가 적혀 있을 뿐입니다.
> 이 프로그램은 그 표시만 지우고 나머지는 그대로 다시 묶습니다.
> 수식, 서식, 차트, 매크로는 건드리지 않습니다.

## 대상 파일

`.xlsx`, `.xlsm`

구형 `.xls` 는 구조가 전혀 달라서 안 됩니다. 엑셀에서 한 번 `.xlsx` 로 저장한 뒤 넣으시면 됩니다.

---

## 만들기 (개발자용)

### 필요한 것

- Rust (stable)
- Windows 에서 빌드하거나, 리눅스에서 만든다면 `mingw-w64`

### 윈도우에서

```bash
cargo build --release
```

### 리눅스에서 윈도우용으로

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

결과물은 `target/.../release/excel_unlock.exe` 입니다.

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
  core.rs      해제 로직 (OS 무관, 여기만 보면 원리를 알 수 있음)
  config.rs    저장 경로 설정 보관
  gui.rs       Win32 화면
  bin/
    coretest.rs  GUI 없이 로직만 돌리는 콘솔 도구
assets/
  app.ico       exe 아이콘
  app.manifest  비주얼 스타일 + DPI 인식
icons/          아이콘 원본(SVG) 및 각 크기 PNG
release/        배포용 exe (압축)
```

### 쓰인 것

| | |
|---|---|
| 언어 | Rust |
| 화면 | [native-windows-gui](https://github.com/gabdube/native-windows-gui) (Win32 직접 호출) |
| 압축 | [zip](https://github.com/zip-rs/zip2) |
| 패턴 | [regex](https://github.com/rust-lang/regex) |

런타임 의존성이 없어서 exe 하나(1.7MB)로 끝납니다.

## 만들면서 걸린 것들

같은 걸 만드실 분을 위해 적어 둡니다.

**1. 빈 보호 태그를 보호로 세면 안 됩니다.**
일부 프로그램은 보호를 걸지 않아도 `<workbookProtection/>` 를 넣습니다. 속성이 하나도 없는 이 태그는 아무것도 잠그지 않습니다. 세지 말고 조용히 지우면 됩니다.

**2. `build.rs` 를 `#[cfg(windows)]` 로 감싸면 안 됩니다.**
빌드 스크립트는 *빌드하는 쪽* 에서 돌아갑니다. 리눅스에서 윈도우용을 만들면 그 블록이 통째로 건너뛰어져, 아이콘도 매니페스트도 없는 exe 가 조용히 나옵니다. 빌드는 성공하므로 알아채기 어렵습니다. `CARGO_CFG_TARGET_OS` 로 판단해야 합니다.

**3. winres 가 만든 리소스는 링커가 버립니다.**
정적 라이브러리로 넘어오는데, 코드에서 참조하는 심볼이 없어서 링커가 통째로 제외합니다. `.o` 를 직접 링크해야 합니다.

```rust
println!("cargo:rustc-link-arg-bins={}/resource.o", out_dir);
```

**4. 별도 최상위 창은 이벤트 핸들러가 따로 필요합니다.**
nwg 의 `full_bind_event_handler` 는 `EnumChildWindows` 로 자식만 훑습니다. 설정 창처럼 자식이 아닌 창은 메인 창에만 핸들러를 걸면 버튼이 전부 먹통이 됩니다.

**5. 건드리지 않는 ZIP 항목은 `raw_copy_file` 로.**
압축을 풀었다 다시 압축하면 느리고, 원본 바이트가 바뀝니다. 그대로 복사하는 게 빠르고 안전합니다.

---

## 라이선스

공개. 누구나 자유롭게 쓰시면 됩니다.

## 만든이

**KIM.H,J** · aibulapia@gmail.com · [51work.bid](https://51work.bid)
