//! Lokked — library entry point.
//!
//! `main.rs` is a thin wrapper around [`run`]; the real entry point lives here
//! because Tauri's mobile targets build this crate as a library.
//!
//! Module layout:
//! - [`core`]     — pure Rust domain logic, no Tauri and no I/O.
//! - [`db`]       — SQLite access and migrations.
//! - [`platform`] — OS-specific services behind one trait.
//! - [`commands`] — the thin `#[tauri::command]` layer bridging Rust and TypeScript.
//!
//! Note: the `core` module shadows Rust's built-in `core` crate inside this
//! crate. Always refer to it as `crate::core::…`, never as a bare `core::…`.

pub mod commands;
pub mod core;
pub mod db;
pub mod platform;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database = db::Database::open(app.handle())?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
