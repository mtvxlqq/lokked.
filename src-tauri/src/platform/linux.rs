//! Linux backend (Fedora / GNOME / Wayland is the reference target).
//!
//! Two things the desktop can do that a study timer needs: keep the screen
//! from blanking while a work phase runs, and say when the machine is about
//! to suspend. Both are D-Bus, and both are best-effort — a session bus that
//! refuses is a worse screensaver, never a broken timer.
//!
//! Sleep inhibition is tried through the desktop portal first, because that
//! is the one path that also works inside a Flatpak sandbox, and falls back
//! to `org.freedesktop.ScreenSaver`, which every desktop environment
//! implements. Suspend notifications come from `logind`, which is on the
//! system bus and has no portal equivalent.
//!
//! The blocking zbus API is used deliberately: [`PlatformServices`] is a
//! synchronous trait called from Tauri commands, and the one long-lived
//! subscription runs on a thread of its own rather than dragging an async
//! runtime into the app.

use std::collections::HashMap;

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedObjectPath, Value};

use super::{PlatformError, PlatformServices, SleepEvent, SleepWatcher};

/// What is shown to the student if the desktop asks why the screen is being
/// kept awake.
const REASON: &str = "Идёт учебная сессия";

/// The portal's flag for «do not idle», the only one this app wants: it
/// keeps the screen from blanking without claiming the machine may never
/// suspend when the lid closes.
const INHIBIT_IDLE: u32 = 8;

/// A held inhibitor, in whichever way it was taken.
#[derive(Debug)]
enum Inhibitor {
    /// A portal request object, released by closing it.
    Portal(OwnedObjectPath),
    /// A screensaver cookie, released by handing it back.
    ScreenSaver(u32),
}

/// Sleep inhibition and suspend notifications via D-Bus.
#[derive(Debug, Default)]
pub struct LinuxPlatform {
    /// Kept alive for as long as an inhibitor is held: the screensaver
    /// cookie is tied to the connection that took it, and dropping the
    /// connection would silently release it.
    session: Option<Connection>,
    held: Option<Inhibitor>,
}

impl LinuxPlatform {
    /// The session bus, opened on first use and kept afterwards.
    fn session(&mut self) -> Result<&Connection, PlatformError> {
        if self.session.is_none() {
            self.session = Some(Connection::session().map_err(backend)?);
        }

        Ok(self.session.as_ref().expect("just opened"))
    }
}

impl PlatformServices for LinuxPlatform {
    fn inhibit_sleep(&mut self) -> Result<(), PlatformError> {
        if self.held.is_some() {
            return Ok(());
        }

        let connection = self.session()?.clone();

        // Портал сначала: он же единственный, что работает внутри Flatpak.
        // Если его нет, остаётся хранитель экрана — его реализуют все.
        self.held = match inhibit_via_portal(&connection) {
            Ok(handle) => Some(Inhibitor::Portal(handle)),
            Err(_) => Some(Inhibitor::ScreenSaver(inhibit_via_screensaver(
                &connection,
            )?)),
        };

        Ok(())
    }

    fn release_sleep(&mut self) -> Result<(), PlatformError> {
        let Some(held) = self.held.take() else {
            // Освобождение без захвата — обычное дело: сессию могли
            // остановить, когда шёл перерыв.
            return Ok(());
        };

        let connection = self.session()?.clone();

        match held {
            Inhibitor::Portal(handle) => {
                Proxy::new(
                    &connection,
                    "org.freedesktop.portal.Desktop",
                    handle.as_str(),
                    "org.freedesktop.portal.Request",
                )
                .map_err(backend)?
                .call::<_, _, ()>("Close", &())
                .map_err(backend)?;
            }
            Inhibitor::ScreenSaver(cookie) => {
                screensaver(&connection)?
                    .call::<_, _, ()>("UnInhibit", &(cookie,))
                    .map_err(backend)?;
            }
        }

        Ok(())
    }

    fn notify(&self, _title: &str, _body: &str) -> Result<(), PlatformError> {
        // Уведомления идут через tauri-plugin-notification: он один и тот же
        // на десктопе и на мобилке, и дублировать его здесь незачем.
        Ok(())
    }

    fn watch_sleep(&mut self, on_event: SleepWatcher) -> Result<(), PlatformError> {
        // Системная шина, а не сессионная: logind живёт только там, и
        // портала для него нет.
        let connection = Connection::system().map_err(backend)?;
        let proxy = Proxy::new(
            &connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
        .map_err(backend)?;

        let signals = proxy.receive_signal("PrepareForSleep").map_err(backend)?;

        // Свой поток на всё время жизни приложения: подписка блокирующая, а
        // соединение переезжает в поток вместе с ней, иначе оно закроется и
        // сигналы перестанут приходить.
        std::thread::Builder::new()
            .name("lokked-sleep-watch".to_string())
            .spawn(move || {
                // Соединение держится живым ровно потому, что оно здесь.
                let _connection = connection;

                for signal in signals {
                    if let Ok(going_to_sleep) = signal.body().deserialize::<bool>() {
                        on_event(sleep_event(going_to_sleep));
                    }
                }
            })
            .map_err(|err| PlatformError::Backend(err.to_string()))?;

        Ok(())
    }
}

/// What `PrepareForSleep(bool)` means: `true` on the way down, `false` on
/// the way back.
fn sleep_event(going_to_sleep: bool) -> SleepEvent {
    if going_to_sleep {
        SleepEvent::GoingToSleep
    } else {
        SleepEvent::WokeUp
    }
}

/// `org.freedesktop.portal.Inhibit` — the sandbox-friendly path.
fn inhibit_via_portal(connection: &Connection) -> Result<OwnedObjectPath, PlatformError> {
    let proxy = Proxy::new(
        connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Inhibit",
    )
    .map_err(backend)?;

    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("reason", Value::from(REASON));

    // Первый аргумент — идентификатор родительского окна; на Wayland его
    // взять неоткуда, и пустая строка здесь допустима по спецификации.
    proxy
        .call("Inhibit", &("", INHIBIT_IDLE, options))
        .map_err(backend)
}

/// `org.freedesktop.ScreenSaver` — the fallback every desktop implements.
fn inhibit_via_screensaver(connection: &Connection) -> Result<u32, PlatformError> {
    screensaver(connection)?
        .call("Inhibit", &("com.lokked.app", REASON))
        .map_err(backend)
}

fn screensaver(connection: &Connection) -> Result<Proxy<'_>, PlatformError> {
    Proxy::new(
        connection,
        "org.freedesktop.ScreenSaver",
        "/org/freedesktop/ScreenSaver",
        "org.freedesktop.ScreenSaver",
    )
    .map_err(backend)
}

fn backend(err: zbus::Error) -> PlatformError {
    PlatformError::Backend(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_signal_says_which_way_the_machine_is_going() {
        assert_eq!(sleep_event(true), SleepEvent::GoingToSleep);
        assert_eq!(sleep_event(false), SleepEvent::WokeUp);
    }

    #[test]
    fn releasing_without_holding_anything_is_not_an_error() {
        // Так и бывает: сессию остановили во время перерыва, когда экран
        // никто не удерживал.
        let mut platform = LinuxPlatform::default();

        assert!(platform.release_sleep().is_ok());
    }
}
