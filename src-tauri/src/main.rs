#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if wandel_lib::try_run_headless_mcp() {
        return;
    }
    wandel_lib::run()
}
