//! Tests for the card side of the statistics screen: accuracy by day, the
//! cards worth going back to, and everything one card's history adds up to.

use lokked_lib::core::review::Grade;
use lokked_lib::core::scheduler::CardAccuracy;
use lokked_lib::core::stats::cards::{
    accuracy_by_day, card_stats, problem_cards, CardAnswer, RECENT_ANSWERS,
};

/// `(day_key, answered, correct)` triples, as they come out of the database.
fn days(rows: &[(&str, u32, u32)]) -> Vec<(String, u32, u32)> {
    rows.iter()
        .map(|(day, answered, correct)| ((*day).to_string(), *answered, *correct))
        .collect()
}

fn accuracy(rows: &[(&str, u32, u32)]) -> Vec<CardAccuracy> {
    rows.iter()
        .map(|(id, shown, correct)| CardAccuracy {
            card_id: (*id).to_string(),
            shown: *shown,
            correct: *correct,
        })
        .collect()
}

/// A card's history, oldest answer first.
fn answers(grades: &[Grade]) -> Vec<CardAnswer> {
    grades
        .iter()
        .map(|grade| CardAnswer {
            grade: *grade,
            think_ms: None,
        })
        .collect()
}

#[test]
fn accuracy_is_reported_for_every_day_of_the_period() {
    let counted = accuracy_by_day(&days(&[("2026-08-20", 4, 3)]), "2026-08-19", "2026-08-20");

    assert_eq!(counted.len(), 2);
    assert_eq!(counted[0].day_key, "2026-08-19");
    assert_eq!(counted[0].answered, 0);
    // День без ответов — это ноль ответов, а не ноль процентов точности:
    // рисовать по нему точку на графике нечего.
    assert_eq!(counted[0].accuracy_percent, 0);
    assert_eq!(counted[1].answered, 4);
    assert_eq!(counted[1].correct, 3);
    assert_eq!(counted[1].accuracy_percent, 75);
}

#[test]
fn accuracy_rounds_to_the_nearest_per_cent() {
    let counted = accuracy_by_day(&days(&[("2026-08-20", 3, 2)]), "2026-08-20", "2026-08-20");

    assert_eq!(counted[0].accuracy_percent, 67);
}

#[test]
fn a_day_answered_wrong_throughout_is_zero_per_cent() {
    let counted = accuracy_by_day(&days(&[("2026-08-20", 5, 0)]), "2026-08-20", "2026-08-20");

    assert_eq!(counted[0].answered, 5);
    assert_eq!(counted[0].accuracy_percent, 0);
}

#[test]
fn repeated_rows_for_one_day_are_summed() {
    let counted = accuracy_by_day(
        &days(&[("2026-08-20", 2, 1), ("2026-08-20", 2, 2)]),
        "2026-08-20",
        "2026-08-20",
    );

    assert_eq!(counted[0].answered, 4);
    assert_eq!(counted[0].correct, 3);
}

#[test]
fn an_unparsable_period_gives_no_days() {
    assert_eq!(
        accuracy_by_day(&days(&[]), "позавчера", "2026-08-20").len(),
        0
    );
}

#[test]
fn the_worst_card_is_listed_first() {
    let listed = problem_cards(&accuracy(&[("easy", 10, 9), ("hard", 10, 2)]), 20, 3);

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].card_id, "hard");
    assert_eq!(listed[0].shown, 10);
    assert_eq!(listed[0].correct, 2);
    assert_eq!(listed[0].accuracy_percent, 20);
    assert_eq!(listed[1].card_id, "easy");
}

#[test]
fn a_card_seen_too_few_times_is_new_rather_than_weak() {
    let listed = problem_cards(&accuracy(&[("fresh", 2, 0), ("known", 5, 4)]), 20, 3);

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].card_id, "known");
}

#[test]
fn the_list_stops_at_the_limit() {
    let stats = accuracy(&[("a", 5, 0), ("b", 5, 1), ("c", 5, 2)]);

    let listed = problem_cards(&stats, 2, 3);

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].card_id, "a");
    assert_eq!(listed[1].card_id, "b");
}

#[test]
fn a_card_nobody_has_answered_has_nothing_to_show() {
    let stats = card_stats(&answers(&[]));

    assert_eq!(stats.shown, 0);
    assert_eq!(stats.correct, 0);
    assert_eq!(stats.accuracy_percent, 0);
    assert_eq!(stats.current_streak, 0);
    assert_eq!(stats.average_think_ms, None);
    assert!(stats.recent.is_empty());
}

#[test]
fn one_answer_is_already_a_hundred_per_cent() {
    let stats = card_stats(&answers(&[Grade::Good]));

    assert_eq!(stats.shown, 1);
    assert_eq!(stats.correct, 1);
    assert_eq!(stats.accuracy_percent, 100);
    assert_eq!(stats.current_streak, 1);
}

#[test]
fn recalling_with_difficulty_still_counts_as_recalling() {
    let stats = card_stats(&answers(&[Grade::Hard]));

    assert_eq!(stats.correct, 1);
    assert_eq!(stats.current_streak, 1);
}

#[test]
fn a_card_missed_every_time_is_zero_per_cent() {
    let stats = card_stats(&answers(&[Grade::Again, Grade::Again]));

    assert_eq!(stats.accuracy_percent, 0);
    assert_eq!(stats.current_streak, 0);
}

#[test]
fn the_streak_counts_back_from_the_last_answer() {
    let stats = card_stats(&answers(&[
        Grade::Good,
        Grade::Again,
        Grade::Good,
        Grade::Easy,
    ]));

    assert_eq!(stats.shown, 4);
    assert_eq!(stats.correct, 3);
    assert_eq!(stats.current_streak, 2);
}

#[test]
fn only_the_last_ten_answers_are_kept_and_the_oldest_of_them_comes_first() {
    let mut history = vec![Grade::Again];
    history.extend(vec![Grade::Good; RECENT_ANSWERS]);

    let stats = card_stats(&answers(&history));

    assert_eq!(stats.recent.len(), RECENT_ANSWERS);
    // Самый первый ответ — «не помню» — уже не влезает в цепочку.
    assert!(stats.recent.iter().all(|grade| *grade == Grade::Good));
}

#[test]
fn the_average_recall_time_ignores_answers_that_were_never_timed() {
    let history = vec![
        CardAnswer {
            grade: Grade::Good,
            think_ms: Some(3000),
        },
        CardAnswer {
            grade: Grade::Good,
            think_ms: None,
        },
        CardAnswer {
            grade: Grade::Good,
            think_ms: Some(2000),
        },
    ];

    assert_eq!(card_stats(&history).average_think_ms, Some(2500));
}
