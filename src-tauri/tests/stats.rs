//! Tests for the streak: how many days in a row the student studied enough.

use lokked_lib::core::stats::{streak, STREAK_MIN_SECONDS};

/// `(day_key, active_seconds)` pairs, as they come out of the database.
fn days(pairs: &[(&str, i64)]) -> Vec<(String, i64)> {
    pairs
        .iter()
        .map(|(day, seconds)| ((*day).to_string(), *seconds))
        .collect()
}

/// A day that counts, and one that does not quite.
const ENOUGH: i64 = STREAK_MIN_SECONDS;
const TOO_LITTLE: i64 = STREAK_MIN_SECONDS - 1;

#[test]
fn a_student_who_has_never_studied_has_no_streak() {
    assert_eq!(streak(&days(&[]), "2026-08-21"), 0);
}

#[test]
fn studying_enough_today_makes_a_streak_of_one() {
    assert_eq!(streak(&days(&[("2026-08-21", ENOUGH)]), "2026-08-21"), 1);
}

#[test]
fn consecutive_days_add_up() {
    let recorded = days(&[
        ("2026-08-19", ENOUGH),
        ("2026-08-20", ENOUGH),
        ("2026-08-21", ENOUGH),
    ]);

    assert_eq!(streak(&recorded, "2026-08-21"), 3);
}

#[test]
fn a_missed_day_ends_the_streak() {
    let recorded = days(&[
        ("2026-08-17", ENOUGH),
        ("2026-08-18", ENOUGH),
        // 19 августа пропущено.
        ("2026-08-20", ENOUGH),
        ("2026-08-21", ENOUGH),
    ]);

    assert_eq!(streak(&recorded, "2026-08-21"), 2);
}

#[test]
fn a_day_below_the_threshold_does_not_count() {
    let recorded = days(&[("2026-08-20", ENOUGH), ("2026-08-21", TOO_LITTLE)]);

    // Сегодня ещё не засчитано, но вчерашняя серия жива — она и показывается.
    assert_eq!(streak(&recorded, "2026-08-21"), 1);
}

#[test]
fn a_streak_survives_a_day_that_has_only_just_begun() {
    // Ключевое свойство: серия не обнуляется в полночь. Пока сегодня пусто,
    // показывается вчерашняя.
    let recorded = days(&[("2026-08-19", ENOUGH), ("2026-08-20", ENOUGH)]);

    assert_eq!(streak(&recorded, "2026-08-21"), 2);
}

#[test]
fn a_streak_that_ended_the_day_before_yesterday_is_over() {
    let recorded = days(&[("2026-08-18", ENOUGH), ("2026-08-19", ENOUGH)]);

    assert_eq!(streak(&recorded, "2026-08-21"), 0);
}

#[test]
fn several_sessions_in_one_day_are_summed_before_the_threshold_applies() {
    let recorded = days(&[("2026-08-21", 4 * 60), ("2026-08-21", 7 * 60)]);

    assert_eq!(streak(&recorded, "2026-08-21"), 1);
}

#[test]
fn days_after_today_are_ignored() {
    // Часы могли уйти вперёд и вернуться; будущее в серию не засчитывается.
    let recorded = days(&[("2026-08-22", ENOUGH), ("2026-08-21", ENOUGH)]);

    assert_eq!(streak(&recorded, "2026-08-21"), 1);
}

#[test]
fn a_streak_runs_across_a_month_boundary() {
    let recorded = days(&[
        ("2026-07-30", ENOUGH),
        ("2026-07-31", ENOUGH),
        ("2026-08-01", ENOUGH),
    ]);

    assert_eq!(streak(&recorded, "2026-08-01"), 3);
}

#[test]
fn an_unreadable_day_key_is_skipped_rather_than_trusted() {
    let recorded = days(&[("не дата", ENOUGH), ("2026-08-21", ENOUGH)]);

    assert_eq!(streak(&recorded, "2026-08-21"), 1);
}

#[test]
fn an_unreadable_today_has_no_streak_to_speak_of() {
    assert_eq!(streak(&days(&[("2026-08-21", ENOUGH)]), "сегодня"), 0);
}
