#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;

fn main() -> Result<()> {
    // Launch the independent modern GUI directly
    live_processor::gui::run_gui()
}
