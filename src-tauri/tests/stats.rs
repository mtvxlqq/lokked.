//! Tests for the pure aggregations: the streak, and the numbers under a
//! finished run of cards.

use lokked_lib::core::review::Grade;
use lokked_lib::core::stats::{
    review_summary, streak, ReviewOutcome, ReviewSummary, STREAK_MIN_SECONDS,
};

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

// --- итоги прогона ---------------------------------------------------------

fn answer(card: &str, grade: Grade, total_ms: i64) -> ReviewOutcome {
    ReviewOutcome {
        card_id: card.to_string(),
        grade,
        total_ms,
    }
}

#[test]
fn a_run_with_no_answers_has_nothing_to_report() {
    assert_eq!(review_summary(&[]), ReviewSummary::default());
}

#[test]
fn everything_recalled_is_a_hundred_per_cent() {
    let results = [
        answer("c-1", Grade::Good, 4000),
        answer("c-2", Grade::Easy, 2000),
        answer("c-3", Grade::Hard, 9000),
    ];

    let summary = review_summary(&results);

    assert_eq!(summary.answered, 3);
    assert_eq!(summary.correct, 3);
    assert_eq!(summary.accuracy_percent, 100);
    assert!(summary.mistakes.is_empty());
}

#[test]
fn accuracy_is_rounded_to_the_nearest_per_cent() {
    let results = [
        answer("c-1", Grade::Good, 1000),
        answer("c-2", Grade::Good, 1000),
        answer("c-3", Grade::Again, 1000),
    ];

    assert_eq!(review_summary(&results).accuracy_percent, 67);
}

#[test]
fn the_mistakes_are_the_cards_answered_again_in_order() {
    let results = [
        answer("c-1", Grade::Again, 1000),
        answer("c-2", Grade::Good, 1000),
        answer("c-3", Grade::Again, 1000),
    ];

    let summary = review_summary(&results);

    assert_eq!(summary.correct, 1);
    assert_eq!(summary.accuracy_percent, 33);
    assert_eq!(summary.mistakes, vec!["c-1", "c-3"]);
}

#[test]
fn the_time_is_the_sum_and_the_mean_of_the_answers() {
    let results = [
        answer("c-1", Grade::Good, 3000),
        answer("c-2", Grade::Good, 4000),
    ];

    let summary = review_summary(&results);

    assert_eq!(summary.total_ms, 7000);
    assert_eq!(summary.average_ms, 3500);
}

#[test]
fn a_single_answer_is_all_of_it() {
    let summary = review_summary(&[answer("c-1", Grade::Again, 12_000)]);

    assert_eq!(summary.answered, 1);
    assert_eq!(summary.accuracy_percent, 0);
    assert_eq!(summary.average_ms, 12_000);
    assert_eq!(summary.mistakes, vec!["c-1"]);
}
