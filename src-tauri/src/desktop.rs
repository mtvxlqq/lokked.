//! The desktop wiring: what a second launch does, and what happens when the
//! machine goes to sleep.
//!
//! On Wayland an app cannot grab a global hotkey itself, so the shortcut is
//! registered in GNOME Settings and runs `lokked --toggle`. The single
//! instance plugin hands that argv to the copy that is already running, and
//! [`handle_cli`] turns it into an action. What the argv *means* is parsed
//! in [`crate::core::cli`]; this module is the part that needs an app to act
//! on.
//!
//! The other half is `logind`: [`watch_sleep`] pauses a running session
//! before the machine suspends and, once it comes back, asks the frontend to
//! offer carrying on. The time asleep is never counted as study.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::session::actions::stop;
use crate::commands::session::desktop::{pause_for_sleep, toggle};
use crate::commands::session::SessionState;
use crate::core::cli::{parse_args, CliCommand};
use crate::db::{backup, Database};
use crate::platform::clock::SystemClock;
use crate::platform::{SharedPlatform, SleepEvent};

/// Asks the frontend to open the black screen.
pub const ZEN_EVENT: &str = "lokked://zen";

/// Tells the frontend the machine woke up and the session is on pause.
pub const WOKE_EVENT: &str = "lokked://woke";

/// How long the machine was asleep, for the dialog that offers to carry on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WokeUp {
    pub asleep_seconds: i64,
}

/// A `--zen` that arrived before there was a window to show it in.
///
/// The very first launch parses its own argv, but the frontend is not
/// listening yet — it has not been built. So the request is remembered here
/// and the frontend asks for it when it mounts.
#[derive(Default)]
pub struct PendingZen(AtomicBool);

impl PendingZen {
    fn set(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    fn take(&self) -> bool {
        self.0.swap(false, Ordering::SeqCst)
    }
}

/// Whether the app was asked to start in the black screen. Answers `true`
/// once, then forgets — a reload of the frontend must not reopen it.
#[tauri::command]
pub fn cli_pending_zen(pending: State<'_, PendingZen>) -> bool {
    pending.take()
}

/// Acts on the argv of a launch — this one or a later one.
///
/// A launch with no flags still means something: the student ran the app
/// again, so the window comes forward instead of a second copy starting.
pub fn handle_cli(app: &AppHandle, args: &[String]) {
    match parse_args(args) {
        Some(CliCommand::Toggle) => {
            let _ = toggle(
                &app.state::<SessionState>(),
                &app.state::<SharedPlatform>(),
                &SystemClock,
            );
        }
        Some(CliCommand::Stop) => {
            let _ = stop(
                &app.state::<Database>(),
                &app.state::<SessionState>(),
                &app.state::<SharedPlatform>(),
                &SystemClock,
            );
        }
        Some(CliCommand::Zen) => {
            focus(app);
            // Окна ещё может не быть — на самом первом запуске фронтенд
            // спросит про это сам.
            if app.emit(ZEN_EVENT, ()).is_err() {
                app.state::<PendingZen>().set();
            }
        }
        None => focus(app),
    }
}

/// The first launch acts on its own command line.
///
/// `--zen` is remembered rather than emitted: the frontend is not listening
/// this early.
pub fn handle_startup_cli(app: &AppHandle) {
    let args: Vec<String> = std::env::args().collect();

    if parse_args(&args) == Some(CliCommand::Zen) {
        app.state::<PendingZen>().set();
        return;
    }

    handle_cli(app, &args);
}

/// Brings the main window forward.
fn focus(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Pauses the session while the machine sleeps, and offers to carry on when
/// it comes back.
pub fn watch_sleep(app: &AppHandle) {
    let handle = app.clone();
    // Момент засыпания, и заодно признак того, что паузу поставили мы: если
    // студент остановил таймер сам, спрашивать его после пробуждения не о
    // чем.
    let asleep_since: Arc<Mutex<Option<DateTime<Utc>>>> = Arc::default();

    app.state::<SharedPlatform>()
        .watch_sleep(Box::new(move |event| match event {
            SleepEvent::GoingToSleep => {
                let paused = pause_for_sleep(
                    &handle.state::<SessionState>(),
                    &handle.state::<SharedPlatform>(),
                    &SystemClock,
                );

                *asleep_since.lock().expect("sleep mutex poisoned") = paused.then(Utc::now);
            }
            SleepEvent::WokeUp => {
                let since = asleep_since.lock().expect("sleep mutex poisoned").take();

                if let Some(since) = since {
                    let _ = handle.emit(
                        WOKE_EVENT,
                        WokeUp {
                            asleep_seconds: (Utc::now() - since).num_seconds().max(0),
                        },
                    );
                }
            }
        }));
}

/// Takes the startup backup of the database.
///
/// Failures are logged and swallowed: not being able to write a copy is not
/// a reason to refuse to open the timer.
pub fn back_up_database(app: &AppHandle) {
    let Ok(dir) = app.path().app_data_dir() else {
        return;
    };

    if let Err(err) = backup::rotate(
        &app.state::<Database>(),
        &dir.join(backup::DIRECTORY),
        Utc::now(),
    ) {
        eprintln!("Lokked: не удалось сделать резервную копию базы: {err}");
    }
}
