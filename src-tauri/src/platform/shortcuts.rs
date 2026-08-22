//! Global hotkeys, where the OS lets an application take them.
//!
//! Windows hands a global shortcut to whoever asks for it, so Lokked asks:
//! pause, black screen and stop, without the window being in front.
//!
//! Wayland does not, and that is not a bug to work around — a compositor
//! refusing to let a background app grab keys is the security model. There
//! the same three actions are reached by launching the app again with a
//! flag (`lokked --toggle`), and the shortcut itself is set up in GNOME's
//! own settings; see the README.
//!
//! The plugin is depended on unconditionally rather than per target: what is
//! platform-specific is which shortcuts are asked for, and keeping the code
//! itself portable means it is compiled and checked on every host instead of
//! only on the one that runs it.

use tauri::{AppHandle, Builder, Wry};
use tauri_plugin_global_shortcut::{Builder as ShortcutBuilder, Shortcut, ShortcutState};

/// What each shortcut does, in the vocabulary the command line already
/// speaks — so a hotkey and `lokked --toggle` end up in the same place.
///
/// `Super` is left alone: on Windows it belongs to the shell, and a study
/// timer has no business fighting it for `Super+S`.
///
/// Public and not gated behind a `cfg` so that a typo in a key name or in a
/// flag is caught by a test on any machine, not only on the one that
/// registers them.
pub const WINDOWS_SHORTCUTS: &[(&str, &str)] = &[
    ("ctrl+alt+p", "--toggle"),
    ("ctrl+alt+z", "--zen"),
    ("ctrl+alt+s", "--stop"),
];

/// What this OS is asked for. Empty everywhere but Windows — see the module
/// docs.
const fn shortcuts() -> &'static [(&'static str, &'static str)] {
    #[cfg(target_os = "windows")]
    {
        WINDOWS_SHORTCUTS
    }
    #[cfg(not(target_os = "windows"))]
    {
        &[]
    }
}

/// Adds the global-shortcut plugin to the app, if this OS has shortcuts to
/// give.
///
/// `on_command` receives the same flag a launch would carry, so the caller
/// has one place that decides what «pause» means.
pub fn install<F>(builder: Builder<Wry>, on_command: F) -> Builder<Wry>
where
    F: Fn(&AppHandle, &str) + Send + Sync + 'static,
{
    let wanted = shortcuts();
    if wanted.is_empty() {
        return builder;
    }

    let plugin = ShortcutBuilder::new().with_shortcuts(wanted.iter().map(|(keys, _)| *keys));

    let plugin = match plugin {
        Ok(plugin) => plugin,
        // Нечитаемое сочетание — это опечатка в константе выше, а не беда
        // студента: приложение запускается без горячих клавиш.
        Err(_) => return builder,
    };

    builder.plugin(
        plugin
            .with_handler(move |app, shortcut, event| {
                // Нажатие, а не отпускание: иначе каждое срабатывание
                // считалось бы дважды.
                if event.state != ShortcutState::Pressed {
                    return;
                }

                if let Some(command) = command_of(shortcut) {
                    on_command(app, command);
                }
            })
            .build(),
    )
}

/// Which command a shortcut stands for.
///
/// Matched by id rather than by comparing keys: the plugin hands back the
/// same shortcut it was given, and its id is derived from the very keys
/// [`shortcuts`] names.
fn command_of(shortcut: &Shortcut) -> Option<&'static str> {
    shortcuts().iter().find_map(|(keys, command)| {
        let parsed: Shortcut = keys.parse().ok()?;

        (parsed.id() == shortcut.id()).then_some(*command)
    })
}
