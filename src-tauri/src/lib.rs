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
//! - [`desktop`] — the desktop wiring: command line, suspend, startup backup.
//!
//! Note: the `core` module shadows Rust's built-in `core` crate inside this
//! crate. Always refer to it as `crate::core::…`, never as a bare `core::…`.

pub mod commands;
pub mod core;
pub mod db;
pub mod desktop;
pub mod platform;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Only one Lokked at a time: a second launch is how the global
        // hotkey talks to the running app (`lokked --toggle`), so its argv
        // is handed over instead of a second window opening.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            desktop::handle_cli(app, &args);
        }))
        // Phase changes are announced by the frontend through this plugin;
        // it is the one official cross-platform way to reach the OS
        // notification centre, on the desktop and on a phone alike.
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let database = db::Database::open(app.handle())?;
            app.manage(database);
            // The active session and the OS services it needs live for as
            // long as the app does; commands reach them through `State`.
            app.manage(commands::session::SessionState::default());
            app.manage(commands::study::StudyState::default());
            app.manage(platform::SharedPlatform::default());
            app.manage(desktop::PendingZen::default());

            // Одна копия базы за запуск, последние семь хранятся.
            desktop::back_up_database(app.handle());
            // Сон машины — не учёба: сессия встаёт на паузу до засыпания.
            desktop::watch_sleep(app.handle());
            // Аргументы собственного запуска: `lokked --zen` из ярлыка или
            // горячей клавиши, когда приложение ещё не было запущено.
            desktop::handle_startup_cli(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            desktop::cli_pending_zen,
            commands::subjects::list_subjects,
            commands::subjects::create_subject,
            commands::subjects::update_subject,
            commands::subjects::delete_subject,
            commands::presets::list_presets,
            commands::presets::create_preset,
            commands::presets::update_preset,
            commands::presets::delete_preset,
            commands::today::today_totals,
            commands::settings::zen_settings,
            commands::settings::set_zen_settings,
            commands::settings::day_settings,
            commands::settings::set_day_settings,
            commands::settings::blitz_settings,
            commands::settings::set_blitz_settings,
            commands::settings::adaptive_settings,
            commands::settings::set_adaptive_settings,
            commands::streak::streak_view,
            commands::streak::streak_save_image,
            commands::streak::streak_settings,
            commands::streak::set_streak_settings,
            commands::decks::list_decks,
            commands::decks::create_deck,
            commands::decks::update_deck,
            commands::decks::delete_deck,
            commands::cards::list_cards,
            commands::cards::create_card,
            commands::cards::update_card,
            commands::cards::move_card,
            commands::cards::delete_card,
            commands::import::preview_import,
            commands::import::import_cards,
            commands::import::export_deck,
            commands::study::actions::study_start,
            commands::study::actions::study_current,
            commands::study::actions::study_reveal,
            commands::study::actions::study_answer,
            commands::study::actions::study_timeout,
            commands::study::actions::study_summary,
            commands::study::actions::study_repeat_mistakes,
            commands::study::actions::study_stop,
            commands::stats::time::stats_time,
            commands::stats::cards::stats_cards,
            commands::stats::cards::stats_card,
            commands::stats::export::stats_export_csv,
            commands::session::actions::start_session,
            commands::session::actions::session_snapshot,
            commands::session::actions::pause_session,
            commands::session::actions::resume_session,
            commands::session::actions::session_mark_interruption,
            commands::session::actions::session_skip_phase,
            commands::session::actions::stop_session,
            commands::session::actions::session_report_return,
            commands::session::actions::session_discard_away,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
