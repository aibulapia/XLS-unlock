// 엑셀 Unlock — Win32 화면
//
// 좌표는 전부 "논리 픽셀"(96 DPI 기준)로 쓴다.
// nwg 가 high-dpi 기능으로 컨트롤 위치·크기·글꼴을 실제 DPI 에 맞춰 키워 준다.
// 그래서 여기서 직접 배율을 곱하면 안 된다. 두 번 커진다.
//
// 글꼴은 반드시 크기를 지정한다. 지정하지 않으면 시스템 기본값이 들어가는데,
// 그 값은 DPI 배율을 안 타서 글자만 칸을 넘치거나 반대로 너무 작아진다.

use native_windows_gui as nwg;
use nwg::{CharFormat, HTextAlign, MessageButtons, MessageChoice, MessageIcons, MessageParams,
          RadioButtonState, VTextAlign};

use crate::config;
use crate::core::{self, Outcome, Patterns, SaveMode};

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const APP_TITLE: &str = "엑셀 Unlock V. 0.7";
const SITE_URL: &str = "https://51work.bid";

/// 처음 열릴 때 크기 (논리 픽셀)
///
/// 가장 긴 글자를 실측해서 정했다.
///   부제목 356px / 로그 한 줄(긴 파일명) 597px / 버튼 4개 합 381px
/// 안쪽 폭 620px 면 로그 긴 줄도 줄바꿈 없이 들어간다.
const W0: i32 = 660;
const H0: i32 = 620;
/// 이보다 작게는 못 줄인다. 더 줄이면 버튼이 겹친다.
const MIN_W: i32 = 560;
const MIN_H: i32 = 460;

// 글꼴 크기(논리 픽셀, 셀 높이). 16 ≈ 9pt
const F_BODY: u32 = 16;
const F_TITLE: u32 = 26;
const F_SMALL: u32 = 15;
const F_DROP: u32 = 19;
/// 로그 글자 크기(pt). CharFormat 은 twips(1pt = 20twips) 를 쓴다.
const LOG_PT: f64 = 10.5;

/// 라벨 배경. 창 배경과 같은 흰색으로 맞춘다.
/// 비워 두면 라벨만 회색으로 떠서 띠처럼 보인다.
const WHITE: [u8; 3] = [255, 255, 255];

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

    f_body: nwg::Font,
    f_title: nwg::Font,
    f_small: nwg::Font,
    f_drop: nwg::Font,

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

struct State {
    cfg: config::Settings,
    patterns: Patterns,
    busy: bool,
}

// ──────────────────────────── 진입점 ────────────────────────────

pub fn run() {
    if let Err(e) = nwg::init() {
        fatal(&format!("초기화에 실패했습니다.\n\n{e}"));
        return;
    }

    let app = Rc::new(RefCell::new(App::default()));
    let state = Rc::new(RefCell::new(State {
        cfg: config::load(),
        patterns: Patterns::new(),
        busy: false,
    }));

    if let Err(e) = build(&app, &state) {
        fatal(&format!("화면을 만들지 못했습니다.\n\n{e}"));
        return;
    }

    bind(&app, &state);
    relayout(&app, W0, H0);
    {
        let a = app.borrow();
        set_window_icons(&a.window);
        set_window_icons(&a.cfg_win);
    }

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

/// 한글이 잘 나오는 글꼴을 순서대로 찾는다
fn pick_family() -> &'static str {
    "맑은 고딕"
}

// ──────────────────────────── 화면 만들기 ────────────────────────────

fn build(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) -> Result<(), nwg::NwgError> {
    // 창 아이콘은 exe 리소스에서 직접 읽는다 (set_window_icons).
    // 여기서 include_bytes 로 또 넣으면 같은 아이콘이 exe 에 두 번 들어간다.
    let bmp_app = nwg::Bitmap::from_bin(include_bytes!("../assets/appicon.bmp")).ok();
    let bmp_51 = nwg::Bitmap::from_bin(include_bytes!("../assets/badge51.bmp")).ok();

    let mut a = app.borrow_mut();
    let fam = pick_family();

    // 글꼴 먼저. 크기를 반드시 지정한다.
    // 지역 변수로 만든다. a 의 글꼴 필드를 빌린 채로 a 의 컨트롤 필드에
    // 쓸 수가 없기 때문이다. 다 만든 뒤 마지막에 a 로 옮긴다.
    let mut f_body = nwg::Font::default();
    let mut f_title = nwg::Font::default();
    let mut f_small = nwg::Font::default();
    let mut f_drop = nwg::Font::default();
    nwg::Font::builder().family(fam).size(F_BODY).build(&mut f_body)?;
    nwg::Font::builder().family(fam).size(F_TITLE).weight(700).build(&mut f_title)?;
    nwg::Font::builder().family(fam).size(F_SMALL).build(&mut f_small)?;
    nwg::Font::builder().family(fam).size(F_DROP).build(&mut f_drop)?;

    // 지정하지 않은 컨트롤도 기본으로 이 글꼴을 쓰게 한다
    let mut global = nwg::Font::default();
    nwg::Font::builder().family(fam).size(F_BODY).build(&mut global)?;
    nwg::Font::set_global_default(Some(global));

    nwg::Window::builder()
        .size((W0, H0))
        .title(APP_TITLE)
        .accept_files(true)
        .center(true)
        .flags(
            nwg::WindowFlags::WINDOW
                | nwg::WindowFlags::MINIMIZE_BOX
                | nwg::WindowFlags::RESIZABLE
                | nwg::WindowFlags::VISIBLE,
        )
        .build(&mut a.window)?;

    // ── 머리말 ──
    nwg::ImageFrame::builder()
        .bitmap(bmp_app.as_ref())
        .background_color(Some([255, 255, 255]))
        .parent(&a.window)
        .build(&mut a.icon_app)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("엑셀 시트보호 해제")
        .font(Some(&f_title))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_title)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("원본은 그대로 두고, '엑셀 Unlock' 폴더를 만들어 그 안에 해제본을 저장합니다.")
        .font(Some(&f_small))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_sub)?;

    nwg::ImageFrame::builder()
        .bitmap(bmp_51.as_ref())
        .background_color(Some([255, 255, 255]))
        .parent(&a.window)
        .build(&mut a.img_51)?;

    nwg::Button::builder()
        .text("?")
        .font(Some(&f_body))
        .parent(&a.window)
        .build(&mut a.btn_help)?;

    // ── 드롭 영역 ──
    nwg::Frame::builder().parent(&a.window).build(&mut a.drop_frame)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("이곳에 엑셀 파일을 끌어다 놓으세요")
        .font(Some(&f_drop))
        .h_align(HTextAlign::Center)
        .v_align(VTextAlign::Center)
        .parent(&a.drop_frame)
        .build(&mut a.lbl_drop1)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("여러 개 한꺼번에 가능 · xlsx, xlsm, xls")
        .font(Some(&f_small))
        .h_align(HTextAlign::Center)
        .v_align(VTextAlign::Center)
        .parent(&a.drop_frame)
        .build(&mut a.lbl_drop2)?;

    // ── 버튼 줄 ──
    nwg::Button::builder()
        .text("파일 선택")
        .font(Some(&f_body))
        .parent(&a.window)
        .build(&mut a.btn_pick)?;
    nwg::Button::builder()
        .text("결과 폴더 열기")
        .font(Some(&f_body))
        .parent(&a.window)
        .build(&mut a.btn_open)?;
    nwg::Button::builder()
        .text("저장 경로 설정")
        .font(Some(&f_body))
        .parent(&a.window)
        .build(&mut a.btn_path)?;
    nwg::Button::builder()
        .text("기록 지우기")
        .font(Some(&f_body))
        .parent(&a.window)
        .build(&mut a.btn_clear)?;

    // ── 저장 위치 ──
    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("저장 위치")
        .font(Some(&f_small))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_path_cap)?;

    let desc = config::describe(&state.borrow().cfg.mode);
    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text(&desc)
        .font(Some(&f_body))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.lbl_path_val)?;

    // ── 로그 ──
    nwg::RichTextBox::builder()
        .readonly(true)
        .parent(&a.window)
        .build(&mut a.log)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("대기 중")
        .font(Some(&f_small))
        .v_align(VTextAlign::Center)
        .parent(&a.window)
        .build(&mut a.status)?;

    // ── 설정 창 ──
    nwg::Window::builder()
        .size((580, 330))
        .title("저장 경로 설정")
        .center(true)
        .flags(nwg::WindowFlags::WINDOW)
        .parent(Some(&a.window))
        .build(&mut a.cfg_win)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("해제된 파일을 어디에 저장할지 정합니다. 설정은 다음 실행에도 유지됩니다.")
        .font(Some(&f_small))
        .size((540, 22))
        .position((22, 18))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_desc)?;

    nwg::RadioButton::builder()
        .text("원본 파일이 있는 폴더")
        .font(Some(&f_body))
        .size((320, 24))
        .position((22, 54))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_r_beside)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("그 폴더 안에 '엑셀 Unlock' 폴더를 만들어 저장합니다.")
        .font(Some(&f_small))
        .size((500, 22))
        .position((46, 78))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_hint1)?;

    nwg::RadioButton::builder()
        .text("지정한 폴더에 모아서")
        .font(Some(&f_body))
        .size((320, 24))
        .position((22, 114))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_r_fixed)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("어떤 파일을 넣든 아래 폴더에만 저장합니다.")
        .font(Some(&f_small))
        .size((500, 22))
        .position((46, 138))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_hint2)?;

    nwg::TextInput::builder()
        .font(Some(&f_body))
        .size((412, 28))
        .position((46, 166))
        .readonly(true)
        .parent(&a.cfg_win)
        .build(&mut a.cfg_path)?;

    nwg::Button::builder()
        .text("찾아보기")
        .font(Some(&f_body))
        .size((92, 28))
        .position((466, 166))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_browse)?;

    nwg::Label::builder()
        .background_color(Some(WHITE))
        .text("같은 이름이 있으면 (2), (3) 번호를 붙여 저장합니다.")
        .font(Some(&f_small))
        .size((330, 22))
        .position((22, 250))
        .v_align(VTextAlign::Center)
        .parent(&a.cfg_win)
        .build(&mut a.cfg_note)?;

    nwg::Button::builder()
        .text("취소")
        .font(Some(&f_body))
        .size((92, 32))
        .position((364, 245))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_cancel)?;

    nwg::Button::builder()
        .text("저장")
        .font(Some(&f_body))
        .size((92, 32))
        .position((466, 245))
        .parent(&a.cfg_win)
        .build(&mut a.cfg_save)?;

    // 글꼴을 App 에 옮겨 담는다.
    // 여기서 보관하지 않으면 함수가 끝날 때 핸들이 풀려 글꼴이 깨진다.
    a.f_body = f_body;
    a.f_title = f_title;
    a.f_small = f_small;
    a.f_drop = f_drop;

    drop(a);

    log_line(app, C_INFO, "파일을 끌어다 놓거나 '파일 선택'을 누르세요.");
    Ok(())
}

// ──────────────────────────── 배치 ────────────────────────────

/// 음수가 되지 않게 u32 로 바꾼다 (set_size 가 u32 를 받는다)
#[inline]
fn u(v: i32) -> u32 {
    v.max(1) as u32
}

/// 창 크기에 맞춰 컨트롤을 다시 놓는다. 단위는 논리 픽셀.
fn relayout(app: &Rc<RefCell<App>>, w: i32, h: i32) {
    let a = app.borrow();

    let w = w.max(MIN_W);
    let h = h.max(MIN_H);

    // 옮기기 "전에" 먼저 더럽다고 표시해 둔다.
    // 옮긴 뒤에 표시하면 옛 자리가 이미 화면에 남아버려 자국이 진다.
    mark_dirty(&a.window);

    const PAD: i32 = 20;
    let inner = w - PAD * 2;

    // 머리말
    a.icon_app.set_position(PAD, 18);
    a.icon_app.set_size(36, 36);

    // 오른쪽 끝의 51work 배지와 ? 버튼
    let help_w = 38;
    let badge_w = 26;
    let help_x = w - PAD - help_w;
    let badge_x = help_x - badge_w - 10;
    a.btn_help.set_position(help_x, 18);
    a.btn_help.set_size(u(help_w), 32);
    a.img_51.set_position(badge_x, 21);
    a.img_51.set_size(u(badge_w), 26);

    let text_x = PAD + 36 + 14;
    let text_w = (badge_x - 12 - text_x).max(120);
    a.lbl_title.set_position(text_x, 14);
    a.lbl_title.set_size(u(text_w), 28);
    a.lbl_sub.set_position(text_x, 44);
    a.lbl_sub.set_size(u(text_w), 22);

    // 드롭 영역
    let drop_y = 80;
    let drop_h = 112;
    a.drop_frame.set_position(PAD, drop_y);
    a.drop_frame.set_size(u(inner), u(drop_h));
    a.lbl_drop1.set_position(10, 28);
    a.lbl_drop1.set_size(u(inner - 20), 30);
    a.lbl_drop2.set_position(10, 62);
    a.lbl_drop2.set_size(u(inner - 20), 24);

    // 버튼 줄.
    //   왼쪽  : 파일을 다루는 것      [파일 선택] [결과 폴더 열기]
    //   오른쪽: 설정·정리             [저장 경로 설정] [기록 지우기]
    // 폭은 글자를 실측해서 맞췄다. 예전처럼 전부 같은 폭으로 두면
    // '파일 선택' 처럼 짧은 글자 옆이 휑하게 빈다.
    let by = drop_y + drop_h + 14;
    let bh = 34;
    const GAP: i32 = 8;

    let (w_pick, w_open, w_path, w_clear) = (86, 112, 112, 96);

    // 왼쪽부터
    a.btn_pick.set_position(PAD, by);
    a.btn_pick.set_size(u(w_pick), u(bh));
    a.btn_open.set_position(PAD + w_pick + GAP, by);
    a.btn_open.set_size(u(w_open), u(bh));

    // 오른쪽 끝에서 거꾸로
    let clear_x = w - PAD - w_clear;
    let path_x = clear_x - GAP - w_path;
    a.btn_path.set_position(path_x, by);
    a.btn_path.set_size(u(w_path), u(bh));
    a.btn_clear.set_position(clear_x, by);
    a.btn_clear.set_size(u(w_clear), u(bh));

    // 저장 위치
    let py = by + bh + 12;
    let cap_w = 86;
    a.lbl_path_cap.set_position(PAD + 2, py);
    a.lbl_path_cap.set_size(u(cap_w), 24);
    a.lbl_path_val.set_position(PAD + 2 + cap_w, py);
    a.lbl_path_val.set_size(u(inner - cap_w - 4), 24);

    // 로그 — 남는 공간을 전부 쓴다
    let log_y = py + 30;
    let status_h = 22;
    let log_h = (h - log_y - status_h - 14).max(60);
    a.log.set_position(PAD, log_y);
    a.log.set_size(u(inner), u(log_h));

    a.status.set_position(PAD + 2, h - status_h - 8);
    a.status.set_size(u(inner), u(status_h));

    // 컨트롤을 옮기면 그 자리에 있던 테두리 자국이 그대로 남는다.
    // 부모만 다시 그려서는 안 지워지고, 자식까지 함께 지우고 다시 그려야 한다.
    repaint_all(&a.window);
}

/// 창 아이콘을 큰 것·작은 것 모두 올린다.
///
/// nwg 의 Window::set_icon 은 WM_SETICON 을 wParam=0(ICON_SMALL) 으로만 보낸다.
/// 테스크바는 ICON_BIG 을 쓰기 때문에, 그것만으로는 테스크바에
/// 윈도우 기본 아이콘(빈 창 모양)이 나온다.
///
/// exe 리소스에 박아 둔 아이콘(ID 1)을 화면 배율에 맞는 크기로 각각 읽어서 올린다.
/// 크기를 지정해서 읽어야 .ico 안의 여러 장 중 맞는 것이 골라진다.
fn set_window_icons(win: &nwg::Window) {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL, IMAGE_ICON,
        LR_DEFAULTCOLOR, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, WM_SETICON,
    };

    /// resource.rc 의 `IDI_APP ICON "app.ico"` 에서 IDI_APP = 1
    const ICON_RESOURCE_ID: usize = 1;

    let Some(hwnd) = win.handle.hwnd() else { return };

    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        if hinst.is_null() {
            return;
        }
        // MAKEINTRESOURCEW: 숫자 ID 를 포인터 자리에 그대로 넣는다
        let name = ICON_RESOURCE_ID as *const u16;

        for (which, cx, cy) in [
            (ICON_BIG, SM_CXICON, SM_CYICON),
            (ICON_SMALL, SM_CXSMICON, SM_CYSMICON),
        ] {
            let h = LoadImageW(
                hinst as _,
                name,
                IMAGE_ICON,
                GetSystemMetrics(cx),
                GetSystemMetrics(cy),
                LR_DEFAULTCOLOR,
            );
            if !h.is_null() {
                SendMessageW(hwnd as _, WM_SETICON, which as usize, h as isize);
            }
        }
    }
}

/// 창 전체를 "다시 그려야 함"으로 표시하고 배경을 지운다 (컨트롤을 옮기기 전)
fn mark_dirty(win: &nwg::Window) {
    use windows_sys::Win32::Graphics::Gdi::{
        RedrawWindow, RDW_ALLCHILDREN, RDW_ERASE, RDW_ERASENOW, RDW_INVALIDATE,
    };
    if let Some(hwnd) = win.handle.hwnd() {
        unsafe {
            RedrawWindow(
                hwnd as _,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_ERASENOW,
            );
        }
    }
}

/// 미뤄 둔 그리기를 지금 끝낸다 (컨트롤을 다 옮긴 뒤)
fn repaint_all(win: &nwg::Window) {
    use windows_sys::Win32::Graphics::Gdi::{
        RedrawWindow, RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    if let Some(hwnd) = win.handle.hwnd() {
        unsafe {
            RedrawWindow(
                hwnd as _,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN | RDW_UPDATENOW,
            );
        }
    }
}

// ──────────────────────────── 이벤트 ────────────────────────────

fn bind(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) {
    // full_bind_event_handler 는 EnumChildWindows 로 자식만 훑는다.
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

            E::OnResize | E::OnWindowMaximize => {
                if handle == a.window.handle {
                    let (w, h) = a.window.size();
                    drop(a);
                    relayout(&app_h, w as i32, h as i32);
                }
            }

            E::OnMinMaxInfo => {
                if handle == a.window.handle {
                    if let nwg::EventData::OnMinMaxInfo(info) = &evt_data {
                        info.set_min_size(MIN_W, MIN_H);
                    }
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
                    drop(a);
                    log_line(&app_h, C_INFO, "파일을 끌어다 놓거나 '파일 선택'을 누르세요.");
                } else if is(&a.btn_open) {
                    let dir = config::folder_to_open(&st_h.borrow().cfg);
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
        .filters("엑셀 파일(*.xlsx;*.xlsm;*.xls)|모든 파일(*.*)")
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
            core::unlock_one(f, &s.cfg.mode, &s.patterns)
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
        set_status(
            app,
            &format!("완료 — {done}개 파일이 '{}' 폴더에 있습니다", core::OUT_DIR_NAME),
        );
    } else {
        set_status(app, "완료 — 해제된 파일이 없습니다");
    }

    state.borrow_mut().busy = false;
}

/// 결과를 만든 폴더를 기억해 둔다.
/// '결과 폴더 열기' 가 다음 실행에서도 동작하도록 설정 파일에도 남긴다.
fn remember_out(state: &Rc<RefCell<State>>, out: &Path) {
    let Some(dir) = out.parent() else { return };

    let changed = {
        let mut s = state.borrow_mut();
        if s.cfg.last_out.as_deref() == Some(dir) {
            false
        } else {
            s.cfg.last_out = Some(dir.to_path_buf());
            true
        }
    };

    // 같은 폴더에 여러 개를 저장할 때 매번 디스크에 쓰지 않는다
    if changed {
        let snapshot = state.borrow().cfg.clone();
        let _ = config::save(&snapshot);
    }
}

// ── 설정 창 ──

fn open_settings(app: &Rc<RefCell<App>>, state: &Rc<RefCell<State>>) {
    let a = app.borrow();
    let mode = state.borrow().cfg.mode.clone();

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

    let save_failed = {
        let mut s = state.borrow_mut();
        s.cfg.mode = new_mode.clone();
        // 저장 위치가 바뀌었으니 '마지막으로 만든 폴더' 는 더 이상 맞지 않는다
        s.cfg.last_out = None;
        config::save(&s.cfg).is_err()
    };

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
    let _: MessageChoice = nwg::modal_message(
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

    // CharFormat 의 height 는 twips(1pt = 20twips).
    // 화면 배율만큼 키워야 다른 글자와 크기가 맞는다.
    let twips = (LOG_PT * 20.0 * nwg::scale_factor()).round() as i32;
    a.log.set_char_format(&CharFormat {
        text_color: Some(color),
        height: Some(twips),
        font_face_name: Some(pick_family().to_string()),
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
