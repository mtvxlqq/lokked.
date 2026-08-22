//! Tests for the global-shortcut table.
//!
//! The shortcuts themselves only work on Windows, but the table is checked
//! everywhere: a typo in a key name or in a flag would otherwise only turn
//! up on someone else's machine, silently, as a hotkey that does nothing.

use lokked_lib::core::cli::{parse_args, CliCommand};
use lokked_lib::platform::shortcuts::WINDOWS_SHORTCUTS;
use tauri_plugin_global_shortcut::Shortcut;

#[test]
fn every_shortcut_is_a_key_combination_the_plugin_understands() {
    for (keys, _) in WINDOWS_SHORTCUTS {
        assert!(
            keys.parse::<Shortcut>().is_ok(),
            "сочетание {keys} не разбирается"
        );
    }
}

#[test]
fn every_shortcut_runs_a_command_the_app_already_has() {
    // Горячая клавиша и запуск с флагом попадают в один и тот же обработчик,
    // поэтому флаг обязан быть из того же словаря.
    for (keys, command) in WINDOWS_SHORTCUTS {
        let argv = vec![String::new(), (*command).to_string()];

        assert!(
            parse_args(&argv).is_some(),
            "сочетание {keys} шлёт неизвестный флаг {command}"
        );
    }
}

#[test]
fn the_three_actions_are_all_there_and_none_is_bound_twice() {
    let commands: Vec<&str> = WINDOWS_SHORTCUTS
        .iter()
        .map(|(_, command)| *command)
        .collect();
    let keys: Vec<&str> = WINDOWS_SHORTCUTS.iter().map(|(keys, _)| *keys).collect();

    assert_eq!(commands, vec!["--toggle", "--zen", "--stop"]);
    assert_eq!(
        parse_args(&[String::new(), "--toggle".to_string()]),
        Some(CliCommand::Toggle)
    );

    let mut unique = keys.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), keys.len(), "одно сочетание занято дважды");
}
