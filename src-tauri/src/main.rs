// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix PATH environment variable for macOS GUI apps
    // This ensures the app inherits the shell's PATH in production builds
    let _ = fix_path_env::fix();

    vlottoidx_lib::run()
}
