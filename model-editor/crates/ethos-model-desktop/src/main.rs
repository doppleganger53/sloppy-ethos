#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod native;
#[cfg(windows)]
mod session;

#[cfg(windows)]
fn main() {
    let paths = std::env::args_os()
        .skip(1)
        .map(std::path::PathBuf::from)
        .collect();
    if let Err(msg) = native::run(paths) {
        native_windows_gui::error_message("ETHOS Model Editor", &msg);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("The desktop editor runs on Windows. Use the ethos-model CLI on this platform.");
    std::process::exit(2);
}
