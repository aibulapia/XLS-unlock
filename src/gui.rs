// 엑셀 Unlock V. 0.5 — Win32 화면
//
// 창 하나 + 설정 창 하나. 레이아웃은 전부 고정 좌표로 둔다.
// (창 크기를 못 바꾸게 해 두었으므로 자동 배치가 필요 없다.)

use native_windows_gui as nwg;
use nwg::{CharFormat, HTextAlign, MessageButtons, MessageChoice, MessageIcons, MessageParams,
          RadioButtonState, VTextAlign};

use crate::config;
use crate::core::{self, Outcome, Patterns, SaveMode};

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const APP_TITLE: &str = "엑셀 Unlock V. 0.5";
const SITE_URL: &str = "https://51work.bid";

const W: i32 = 760;
const H: i32 = 600;

// 로그 색
const C_OK: [u8; 3] = [26, 127, 55]; // 초록 - 해제 완료
const C_NONE: [u8; 3] = [106, 106, 106]; // 회색 - 보호 없음 / 건너뜀
const C_LOCK: [u8; 3] = [179, 92, 0]; // 주황 - 열기 암호
const C_FAIL: [u8; 3] = [198, 40, 40]; // 빨강 - 실패
const C_INFO: [u8; 3] = [45, 45, 45]; // 검정 - 안내

// ──────────────────────────── 창 구조 ────────────────────────────

#[derive(Default)]
pub struct App {
    window: nwg::Window,

    icon_app: nwg::ImageFrame,
    lbl_title: nwg::Label,
    lbl_sub: nwg::Label,
    img_51: nwg::ImageFrame,
    btn_help: nwg::Button,

    drop_frame: nwg::Frame,
    lbl_drop1: nwg::Label,
    lbl_drop2: nwg::Label,

    btn_pick: nwg::Button,
    btn_open: nwg::Button,
    btn_path: nwg::Button,
    btn_clear: nwg::Button,

    lbl_path_cap: nwg::Label,
    lbl_path_val: nwg::Label,

    log: nwg::RichTextBox,
    status: nwg::Label,

    // 설정 창
    cfg_win: nwg::Window,
    cfg_desc: nwg::Label,
    cfg_r_beside: nwg::RadioButton,
    cfg_r_fixed: nwg::RadioButton,
    cfg_hint1: nwg::Label,
    cfg_hint2: nwg::Label,
    cfg_path: nwg::TextInput,
    cfg_browse: nwg::Button,
    cfg_note: nwg::Label,
    cfg_cancel: nwg::Button,
    cfg_save: nwg::Button,
}

/// 실행 중에 바뀌는 상태
struct State {
    mode: SaveMode,
    patterns: Patterns,
    busy: bool,
    last_out: Option<PathBuf>,
}

// ──────────────────────────── 진입점 ────────────────────────────

pub fn run() {
    if let Err(e) = nwg::init() {
        fatal(&format!("초기화에 실패했습니다.\n\n{e}"));
        return;
    }
    // 한글이 잘 나오는 글꼴. 없으면 시스템 기본으로 떨어진다.
    let _ = nwg::Font::set_global_family("맑은 고딕");

    let app = Rc::new(RefCell::new(App::default()));
    let state = Rc::new(RefCell::new(State {
        mode: config::load(),
        patterns: Patterns::new(),
        busy: false,
        last_out: None,
    }));

    if let Err(e) = build(&app, &state) {
        fatal(&format!("화면을 만들지 못했습니다.\n\n{e}"));
        return;
    }

    bind(&app, &state);

    // exe 아이콘 위로 파일을 끌어다 놓은 경우
    let argv: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .collect();
    if !argv.is_empty() {
        process(&app, &state, argv);
    }

    nwg::dispatch_thread_events();
}

fn fatal(msg: &str) {
    nwg::message(&MessageParams {
        title: APP_TITLE,
        content: msg,
        buttons: MessageButtons::Ok,
        icons: MessageIcons::Error,
    });
}

// ──────────────────────────── 화면 만들기 ────────────────────────────

fn build(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) -> Result<(), nwg::NwgError> {
    // 리소스는 창보다 먼저 만들어야 한다
    let app_icon = nwg::Icon::from_bin(include_bytes!("../assets/app.ico")).ok();
    let bmp_app = nwg::Bitmap::from_bin(include_bytes!("../assets/appicon.bmp")).ok();
    let bmp_51 = nwg::Bitmap::from_bin(include_bytes!("../assets/badge51.bmp")).ok();

    let mut a = app.borrow_mut();

    nwg::Window::builder()
        .size((W, H))
        .title(APP_TITLE)
        .icon(app_icon.as_ref())
        .accept_files(true)
        .center(true)
        // 크기 조절·최대화를 막는다. 레이아웃이 고정 좌표라 늘어나면 어긋난다.
        .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::MINIMIZE_BOX | nwg::WindowFlags::VISIBLE)
        .build(&mut a.window)?;

    // ── 머리말 ──
    nwg::ImageFrame::builder()
        .size((36, 36))
        .position((20, 18))
        .bitmap(bmp_app.as_ref())
        .background_color(Some([255, 255, 255]))
        .parent(&a.window)
        .build(&mut a.icon_app)?;

    nwg::Label::builder()
        .text("엑셀 시트보호 해제")
        .size((420, 26))
        .position((66, 16))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_title)?;

    nwg::Label::builder()
        .text("원본은 그대로 두고, '엑셀 Unlock' 폴더를 만들어 그 안에 해제본을 저장합니다.")
        .size((560, 20))
        .position((66, 42))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_sub)?;

    nwg::ImageFrame::builder()
        .size((24, 24))
        .position((W - 92, 22))
        .bitmap(bmp_51.as_ref())
        .background_color(Some([255, 255, 255]))
        .parent(&a.window)
        .build(&mut a.img_51)?;

    nwg::Button::builder()
        .text("?")
        .size((34, 28))
        .position((W - 58, 20))
        .parent(&a.window)
        .build(&mut a.btn_help)?;

    // ── 드롭 영역 ──
    nwg::Frame::builder()
        .size((W - 40, 120))
        .position((20, 76))
        .parent(&a.window)
        .build(&mut a.drop_frame)?;

    nwg::Label::builder()
        .text("이곳에 엑셀 파일을 끌어다 놓으세요")
        .size((W - 80, 24))
        .position((20, 36))
        .h_align(HTextAlign::Center)
        .v_align(VTextAlign::Center)
        .parent(&a.drop_frame)
        .build(&mut a.lbl_drop1)?;

    nwg::Label::builder()
        .text("여러 개 한꺼번에 가능 · xlsx, xlsm")
        .size((W - 80, 20))
        .position((20, 62))
        .h_align(HTextAlign::Center)
        .v_align(VTextAlign::Center)
        .parent(&a.drop_frame)
        .build(&mut a.lbl_drop2)?;

    // ── 버튼 줄 ──
    let by = 208;
    nwg::Button::builder()
        .text("파일 선택")
        .size((96, 32))
        .position((20, by))
        .parent(&a.window)
        .build(&mut a.btn_pick)?;

    nwg::Button::builder()
        .text("결과 폴더 열기")
        .size((112, 32))
        .position((124, by))
        .parent(&a.window)
        .build(&mut a.btn_open)?;

    nwg::Button::builder()
        .text("저장 경로 설정")
        .size((112, 32))
        .position((244, by))
        .parent(&a.window)
        .build(&mut a.btn_path)?;

    nwg::Button::builder()
        .text("기록 지우기")
        .size((96, 32))
        .position((W - 116, by))
        .parent(&a.window)
        .build(&mut a.btn_clear)?;

    // ── 저장 위치 표시 ──
    nwg::Label::builder()
        .text("저장 위치")
        .size((64, 22))
        .position((22, 252))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_path_cap)?;

    let desc = config::describe(&state.borrow().mode);
    nwg::Label::builder()
        .text(&desc)
        .size((W - 110, 22))
        .position((88, 252))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_path_val)?;

    // ── 로그 ──
    nwg::RichTextBox::builder()
        .size((W - 40, 272))
        .position((20, 282))
        .readonly(true)
        .parent(&a.window)
        .build(&mut a.log)?;

    nwg::Label::builder()
        .text("대기 중")
        .size((W - 40, 20))
        .position((20, 562))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.status)?;

    // ── 설정 창 (처음엔 숨김) ──
    nwg::Window::builder()
        .size((560, 310))
        .title("저장 경로 설정")
        .icon(app_icon.as_ref())
        .center(true)
        .flags(nwg::WindowFlags::WINDOW)
        .parent(Some(&a.window))
        .build(&mut a.cfg_win)?;

    nwg::Label::builder()
        .text("해제된 파일을 어디에 저장할지 정합니다. 설정은 다음 실행에도 유지됩니다.")
        .size((520, 20))
        .position((22, 18))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_desc)?;

    nwg::RadioButton::builder()
        .text("원본 파일이 있는 폴더")
        .size((300, 22))
        .position((22, 52))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_r_beside)?;

    nwg::Label::builder()
        .text("그 폴더 안에 '엑셀 Unlock' 폴더를 만들어 저장합니다.")
        .size((480, 20))
        .position((44, 74))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_hint1)?;

    nwg::RadioButton::builder()
        .text("지정한 폴더에 모아서")
        .size((300, 22))
        .position((22, 108))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_r_fixed)?;

    nwg::Label::builder()
        .text("어떤 파일을 넣든 아래 폴더에만 저장합니다.")
        .size((480, 20))
        .position((44, 130))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_hint2)?;

    nwg::TextInput::builder()
        .size((400, 26))
        .position((44, 156))
        .readonly(true)
        .parent(&a.cfg_win)
        .build(&mut a.cfg_path)?;

    nwg::Button::builder()
        .text("찾아보기")
        .size((84, 26))
        .position((452, 156))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_browse)?;

    nwg::Label::builder()
        .text("같은 이름이 있으면 (2), (3) 번호를 붙여 저장합니다.")
        .size((320, 20))
        .position((22, 232))
        .v_align(VTextAlign::Center)
        .parent(&a.cfg_win)
        .build(&mut a.cfg_note)?;

    nwg::Button::builder()
        .text("취소")
        .size((84, 30))
        .position((352, 228))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_cancel)?;

    nwg::Button::builder()
        .text("저장")
        .size((84, 30))
        .position((448, 228))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_save)?;

    drop(a);

    log_line(app, C_INFO, "파일을 끌어다 놓거나 '파일 선택'을 누르세요.");
    Ok(())
}

// ──────────────────────────── 이벤트 ────────────────────────────

fn bind(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) {
    // 주의: full_bind_event_handler 는 EnumChildWindows 로 자식만 훑는다.
    // 설정 창은 자식이 아니라 별도 최상위 창이라서 메인 창에만 걸면
    // 설정 창 버튼이 아무 반응도 하지 않는다. 그래서 두 번 건다.
    let (main_handle, cfg_handle) = {
        let a = app.borrow();
        (a.window.handle, a.cfg_win.handle)
    };

    nwg::full_bind_event_handler(&main_handle, make_handler(app, state));
    nwg::full_bind_event_handler(&cfg_handle, make_handler(app, state));
}

fn make_handler(
    app: &Rc<RefCell<App>>,
    state: &Rc<RefCell<State>>,
) -> impl Fn(nwg::Event, nwg::EventData, nwg::ControlHandle) {
    let app_h = app.clone();
    let st_h = state.clone();

    move |evt, evt_data: nwg::EventData, handle| {
        use nwg::Event as E;

        let a = app_h.borrow();
        let is = |c: &dyn HasHandle| c.handle_eq(handle);

        match evt {
            E::OnWindowClose => {
                if handle == a.cfg_win.handle {
                    // 설정 창은 닫는 게 아니라 숨기기만 한다.
                    // 진짜로 닫아버리면 다음에 열 때 창이 없어서 아무 일도 안 일어난다.
                    if let nwg::EventData::OnWindowClose(d) = &evt_data {
                        d.close(false);
                    }
                    drop(a);
                    close_settings(&app_h);
                } else if handle == a.window.handle {
                    drop(a);
                    nwg::stop_thread_dispatch();
                }
            }

            E::OnFileDrop => {
                if handle == a.window.handle || handle == a.drop_frame.handle {
                    let files: Vec<PathBuf> = evt_data
                        .on_file_drop()
                        .files()
                        .into_iter()
                        .map(PathBuf::from)
                        .collect();
                    drop(a);
                    process(&app_h, &st_h, files);
                }
            }

            E::OnImageFrameClick => {
                if is(&a.img_51) {
                    open_external(SITE_URL);
                }
            }

            E::OnButtonClick => {
                if is(&a.btn_help) {
                    drop(a);
                    show_about(&app_h);
                } else if is(&a.btn_pick) {
                    drop(a);
                    pick_files(&app_h, &st_h);
                } else if is(&a.btn_clear) {
                    a.log.clear();
                    a.status.set_text("대기 중");
                } else if is(&a.btn_open) {
                    let dir = config::last_out_dir(&st_h.borrow().mode, st_h.borrow().last_out.as_deref());
                    match dir {
                        Some(d) if d.is_dir() => open_external(&d.to_string_lossy()),
                        _ => {
                            drop(a);
                            info_box(&app_h, "아직 결과 폴더가 없습니다.\n파일을 먼저 처리해 주세요.");
                        }
                    }
                } else if is(&a.btn_path) {
                    drop(a);
                    open_settings(&app_h, &st_h);
                } else if is(&a.cfg_browse) {
                    drop(a);
                    browse_folder(&app_h);
                } else if is(&a.cfg_cancel) {
                    drop(a);
                    close_settings(&app_h);
                } else if is(&a.cfg_save) {
                    drop(a);
                    save_settings(&app_h, &st_h);
                }
            }

            _ => {}
        }
    }
}

/// 핸들 비교를 짧게 쓰기 위한 도우미
trait HasHandle {
    fn handle_eq(&self, h: nwg::ControlHandle) -> bool;
}

macro_rules! impl_has_handle {
    ($($t:ty),*) => {
        $(impl HasHandle for $t {
            fn handle_eq(&self, h: nwg::ControlHandle) -> bool { self.handle == h }
        })*
    };
}
impl_has_handle!(nwg::Button, nwg::ImageFrame, nwg::Window, nwg::Frame, nwg::RadioButton);

// ──────────────────────────── 동작 ────────────────────────────

fn pick_files(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) {
    let mut dlg = nwg::FileDialog::default();
    let built = nwg::FileDialog::builder()
        .title("보호를 해제할 엑셀 파일 선택 (여러 개 가능)")
        .action(nwg::FileDialogAction::Open)
        .multiselect(true)
        .filters("엑셀 파일(*.xlsx;*.xlsm)|모든 파일(*.*)")
        .build(&mut dlg);

    if built.is_err() {
        info_box(app, "파일 선택 창을 열지 못했습니다.");
        return;
    }

    let parent = app.borrow().window.handle;
    if !dlg.run(Some(parent)) {
        return;
    }
    let Ok(items) = dlg.get_selected_items() else {
        return;
    };
    let files: Vec<PathBuf> = items.into_iter().map(PathBuf::from).collect();
    if !files.is_empty() {
        process(app, state, files);
    }
}

fn process(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>, files: Vec<PathBuf>) {
    if files.is_empty() {
        return;
    }
    // 처리 중에 또 떨어뜨려도 꼬이지 않게 막는다
    {
        let mut s = state.borrow_mut();
        if s.busy {
            return;
        }
        s.busy = true;
    }

    set_status(app, &format!("{}개 파일 처리 중...", files.len()));
    log_line(app, C_INFO, &format!("\r\n── {}개 파일 처리 시작 ──", files.len()));

    let mut done = 0usize;
    for f in &files {
        let out = {
            let s = state.borrow();
            core::unlock_one(f, &s.mode, &s.patterns)
        };

        match &out {
            Outcome::Unlocked { out, removed } => {
                done += 1;
                remember_out(state, out);
                log_line(
                    app,
                    C_OK,
                    &format!("  {} — 보호 {removed}개 해제 완료", core::file_name_of(out)),
                );
            }
            Outcome::NoProtection { out } => {
                remember_out(state, out);
                log_line(
                    app,
                    C_NONE,
                    &format!(
                        "  {} — 걸린 보호가 없었습니다 (사본만 생성)",
                        core::file_name_of(out)
                    ),
                );
            }
            Outcome::Encrypted => log_line(
                app,
                C_LOCK,
                &format!(
                    "  {} — 열기 암호가 걸린 파일입니다 (암호 없이는 불가)",
                    core::file_name_of(f)
                ),
            ),
            Outcome::Skipped(m) => log_line(app, C_NONE, &format!("  {m}")),
            Outcome::Failed(m) => log_line(app, C_FAIL, &format!("  {m}")),
        }
    }

    log_line(
        app,
        C_INFO,
        &format!("── 완료 : {done}/{}개 해제됨 ──", files.len()),
    );

    if done > 0 {
        set_status(app, &format!("완료 — {done}개 파일이 '{}' 폴더에 있습니다", core::OUT_DIR_NAME));
    } else {
        set_status(app, "완료 — 해제된 파일이 없습니다");
    }

    state.borrow_mut().busy = false;
}

fn remember_out(state: &Rc<RefCell<State>>, out: &Path) {
    if let Some(d) = out.parent() {
        state.borrow_mut().last_out = Some(d.to_path_buf());
    }
}

// ── 설정 창 ──

fn open_settings(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) {
    let a = app.borrow();
    let mode = state.borrow().mode.clone();

    match &mode {
        SaveMode::BesideSource => {
            a.cfg_r_beside.set_check_state(RadioButtonState::Checked);
            a.cfg_r_fixed.set_check_state(RadioButtonState::Unchecked);
            a.cfg_path.set_text("");
        }
        SaveMode::Fixed(p) => {
            a.cfg_r_beside.set_check_state(RadioButtonState::Unchecked);
            a.cfg_r_fixed.set_check_state(RadioButtonState::Checked);
            a.cfg_path.set_text(&p.to_string_lossy());
        }
    }

    a.window.set_enabled(false);
    a.cfg_win.set_visible(true);
    a.cfg_win.set_focus();
}

fn close_settings(app: &Rc<RefCell<App>>) {
    let a = app.borrow();
    a.cfg_win.set_visible(false);
    a.window.set_enabled(true);
    a.window.set_focus();
}

fn browse_folder(app: &Rc<RefCell<App>>) {
    let mut dlg = nwg::FileDialog::default();
    if nwg::FileDialog::builder()
        .title("결과를 저장할 폴더 선택")
        .action(nwg::FileDialogAction::OpenDirectory)
        .build(&mut dlg)
        .is_err()
    {
        return;
    }

    let parent = app.borrow().cfg_win.handle;
    if !dlg.run(Some(parent)) {
        return;
    }
    if let Ok(item) = dlg.get_selected_item() {
        let a = app.borrow();
        a.cfg_path.set_text(&item.to_string_lossy());
        // 폴더를 고르면 '지정한 폴더' 쪽을 자동으로 선택해 준다
        a.cfg_r_fixed.set_check_state(RadioButtonState::Checked);
        a.cfg_r_beside.set_check_state(RadioButtonState::Unchecked);
    }
}

fn save_settings(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) {
    let picked_fixed = {
        let a = app.borrow();
        a.cfg_r_fixed.check_state() == RadioButtonState::Checked
    };

    let new_mode = if picked_fixed {
        let text = app.borrow().cfg_path.text();
        let p = PathBuf::from(text.trim());
        if p.as_os_str().is_empty() {
            info_box(app, "폴더를 선택해 주세요.\n'찾아보기'를 눌러 고르시면 됩니다.");
            return;
        }
        if !p.is_dir() {
            info_box(app, "그 폴더를 찾을 수 없습니다.\n다시 선택해 주세요.");
            return;
        }
        SaveMode::Fixed(p)
    } else {
        SaveMode::BesideSource
    };

    let save_failed = config::save(&new_mode).is_err();

    {
        let mut s = state.borrow_mut();
        s.mode = new_mode.clone();
        s.last_out = None;
    }

    app.borrow().lbl_path_val.set_text(&config::describe(&new_mode));
    close_settings(app);

    if save_failed {
        info_box(
            app,
            "설정은 이번 실행에만 적용됩니다.\n설정 파일을 저장하지 못했습니다.",
        );
    }
}

// ── 정보 창 ──

fn show_about(app: &Rc<RefCell<App>>) {
    let msg = format!(
        "{APP_TITLE}\r\n\r\n\
         만든이　KIM.H,J\r\n\
         aibulapia@gmail.com\r\n\r\n\
         프로그램 — 공개. 누구나 사용 가능합니다.\r\n\r\n\
         {SITE_URL}"
    );
    let parent = app.borrow().window.handle;
    nwg::modal_message(
        parent,
        &MessageParams {
            title: "프로그램 정보",
            content: &msg,
            buttons: MessageButtons::Ok,
            icons: MessageIcons::Info,
        },
    );
}

fn info_box(app: &Rc<RefCell<App>>, msg: &str) {
    let parent = app.borrow().window.handle;
    let _: MessageChoice = nwg::modal_message(
        parent,
        &MessageParams {
            title: APP_TITLE,
            content: msg,
            buttons: MessageButtons::Ok,
            icons: MessageIcons::Info,
        },
    );
}

// ── 로그 / 상태 ──

fn log_line(app: &Rc<RefCell<App>>, color: [u8; 3], text: &str) {
    let a = app.borrow();
    let end = a.log.len();
    a.log.set_selection(end..end);
    a.log.set_char_format(&CharFormat {
        text_color: Some(color),
        height: Some(190), // twips ≒ 9.5pt
        font_face_name: Some("맑은 고딕".to_string()),
        ..Default::default()
    });
    a.log.append(&format!("{text}\r\n"));
    a.log.scroll_lastline();
}

fn set_status(app: &Rc<RefCell<App>>, text: &str) {
    app.borrow().status.set_text(text);
}

// ── 외부 열기 (링크 / 폴더) ──

fn open_external(target: &str) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide: Vec<u16> = std::ffi::OsStr::new(target)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let verb: Vec<u16> = std::ffi::OsStr::new("open")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            wide.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL as i32,
        );
    }
}
