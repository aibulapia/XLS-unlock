// 엑셀 Unlock V. 0.6
// 창 프로그램이므로 콘솔 창을 띄우지 않는다 (디버그 빌드에서는 로그를 보려고 남겨둔다)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod core;
mod xls;

#[cfg(windows)]
mod gui;

fn main() {
    #[cfg(windows)]
    gui::run();

    #[cfg(not(windows))]
    eprintln!("이 프로그램은 Windows 전용입니다. 해제 로직 확인은 coretest 를 쓰세요.");
}
