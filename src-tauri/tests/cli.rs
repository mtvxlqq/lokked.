//! Tests for the command line: what `lokked --zen` means when the app is
//! already running.

use lokked_lib::core::cli::{parse_args, CliCommand};

/// Argv as the OS hands it over: the binary path first.
fn argv(args: &[&str]) -> Vec<String> {
    std::iter::once("/usr/bin/lokked")
        .chain(args.iter().copied())
        .map(str::to_string)
        .collect()
}

#[test]
fn a_plain_launch_asks_for_nothing() {
    assert_eq!(parse_args(&argv(&[])), None);
}

#[test]
fn the_three_flags_are_recognised() {
    assert_eq!(parse_args(&argv(&["--toggle"])), Some(CliCommand::Toggle));
    assert_eq!(parse_args(&argv(&["--zen"])), Some(CliCommand::Zen));
    assert_eq!(parse_args(&argv(&["--stop"])), Some(CliCommand::Stop));
}

#[test]
fn an_unknown_flag_is_ignored_rather_than_fatal() {
    // Аргументы приходят и от рабочего стола, и от самого GNOME при
    // автозапуске; падать из-за незнакомого флага приложению нельзя.
    assert_eq!(parse_args(&argv(&["--gapplication-service"])), None);
}

#[test]
fn the_first_recognised_flag_wins() {
    assert_eq!(
        parse_args(&argv(&["--zen", "--stop"])),
        Some(CliCommand::Zen)
    );
}

#[test]
fn a_flag_after_something_unknown_is_still_found() {
    assert_eq!(
        parse_args(&argv(&["--verbose", "--stop"])),
        Some(CliCommand::Stop)
    );
}

#[test]
fn the_binary_path_is_never_mistaken_for_a_flag() {
    // Путь к бинарю может быть каким угодно, включая «--zen» в имени
    // каталога; первый элемент argv не разбирается вовсе.
    let args = vec!["/home/student/--zen/lokked".to_string()];

    assert_eq!(parse_args(&args), None);
}

#[test]
fn empty_argv_does_not_panic() {
    assert_eq!(parse_args(&[]), None);
}

#[test]
fn every_command_has_a_flag_and_they_round_trip() {
    for command in [CliCommand::Toggle, CliCommand::Zen, CliCommand::Stop] {
        assert_eq!(parse_args(&argv(&[command.flag()])), Some(command));
    }
}
