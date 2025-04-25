#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
lint::include_modules!();

fn main() {
    println!("Hello, world from the other window!");
}
