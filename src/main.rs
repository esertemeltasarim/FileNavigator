#![windows_subsystem = "windows"]

mod encryption;
mod gui;
mod i18n;

fn main() {
    // Initialize the GUI application
    gui::run_app();
}
