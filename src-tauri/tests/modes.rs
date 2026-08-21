//! Tests for what tells the four modes apart: how many cards each deals,
//! which cards «слабые» picks, and how blitz counts points.

use lokked_lib::core::review::Grade;
use lokked_lib::core::scheduler::{weakest, CardAccuracy, StudyMode, WEAK_LIMIT, WEAK_MIN_SHOWS};
use lokked_lib::core::stats::{blitz_score, ReviewOutcome, BLITZ_POINTS};

fn seen(card: &str, shown: u32, correct: u32) -> CardAccuracy {
    CardAccuracy {
        card_id: card.to_string(),
        shown,
        correct,
    }
}

fn answer(grade: Grade) -> ReviewOutcome {
    ReviewOutcome {
        card_id: "c".to_string(),
        grade,
        total_ms: 1000,
    }
}

// --- режимы ----------------------------------------------------------------

#[test]
fn a_mode_survives_a_round_trip_through_its_slug() {
    for mode in [
        StudyMode::Classic,
        StudyMode::Blitz,
        StudyMode::Marathon,
        StudyMode::Weak,
    ] {
        assert_eq!(StudyMode::parse(mode.as_str()), Ok(mode));
    }
}

#[test]
fn an_unknown_mode_is_refused() {
    assert!(StudyMode::parse("зубрёжка").is_err());
}

#[test]
fn the_marathon_is_the_only_mode_that_takes_the_whole_deck() {
    assert_eq!(StudyMode::Marathon.limit(), None);
    assert!(StudyMode::Classic.limit().is_some());
    assert!(StudyMode::Blitz.limit().is_some());
    assert!(StudyMode::Weak.limit().is_some());
}

#[test]
fn only_blitz_runs_against_a_clock() {
    assert!(StudyMode::Blitz.is_timed());
    for mode in [StudyMode::Classic, StudyMode::Marathon, StudyMode::Weak] {
        assert!(!mode.is_timed());
    }
}

// --- выбор слабых ----------------------------------------------------------

#[test]
fn the_weakest_come_first() {
    let stats = [
        seen("уверенная", 10, 9),
        seen("шаткая", 10, 3),
        seen("средняя", 10, 6),
    ];

    let picked = weakest(&stats, WEAK_LIMIT, WEAK_MIN_SHOWS);

    assert_eq!(picked, vec!["шаткая", "средняя", "уверенная"]);
}

#[test]
fn a_card_shown_too_few_times_is_not_judged_yet() {
    // Одна ошибка из одного показа — это не «слабая карточка», это первый
    // раз: статистики ещё нет.
    let stats = [seen("новая", 1, 0), seen("знакомая", 5, 2)];

    let picked = weakest(&stats, WEAK_LIMIT, WEAK_MIN_SHOWS);

    assert_eq!(picked, vec!["знакомая"]);
}

#[test]
fn nothing_is_picked_when_nothing_has_been_shown_enough() {
    let stats = [seen("новая", 2, 0), seen("тоже новая", 1, 1)];

    assert!(weakest(&stats, WEAK_LIMIT, WEAK_MIN_SHOWS).is_empty());
    assert!(weakest(&[], WEAK_LIMIT, WEAK_MIN_SHOWS).is_empty());
}

#[test]
fn no_more_than_the_limit_is_picked() {
    let stats: Vec<CardAccuracy> = (0..40)
        .map(|n| seen(&format!("card-{n:02}"), 5, 1))
        .collect();

    assert_eq!(
        weakest(&stats, WEAK_LIMIT, WEAK_MIN_SHOWS).len(),
        WEAK_LIMIT
    );
    assert_eq!(weakest(&stats, 3, WEAK_MIN_SHOWS).len(), 3);
}

#[test]
fn cards_with_the_same_accuracy_are_ordered_by_how_often_they_were_seen() {
    // При равной точности вперёд идёт та, по которой данных больше: её
    // слабость подтверждена лучше.
    let stats = [seen("редкая", 3, 1), seen("частая", 30, 10)];

    assert_eq!(
        weakest(&stats, WEAK_LIMIT, WEAK_MIN_SHOWS),
        vec!["частая", "редкая"]
    );
}

#[test]
fn the_order_does_not_depend_on_the_order_the_rows_arrived_in() {
    let one = [seen("a", 5, 1), seen("b", 5, 1)];
    let two = [seen("b", 5, 1), seen("a", 5, 1)];

    assert_eq!(
        weakest(&one, WEAK_LIMIT, WEAK_MIN_SHOWS),
        weakest(&two, WEAK_LIMIT, WEAK_MIN_SHOWS)
    );
}

// --- счёт блица ------------------------------------------------------------

#[test]
fn a_correct_answer_is_worth_the_base_points() {
    let score = blitz_score(&[answer(Grade::Good)]);

    assert_eq!(score.points, BLITZ_POINTS);
    assert_eq!(score.best_streak, 1);
}

#[test]
fn a_miss_is_worth_nothing_and_breaks_the_streak() {
    let results = [
        answer(Grade::Good),
        answer(Grade::Again),
        answer(Grade::Good),
    ];

    let score = blitz_score(&results);

    assert_eq!(score.points, BLITZ_POINTS * 2);
    assert_eq!(score.best_streak, 1);
}

#[test]
fn five_in_a_row_pay_one_and_a_half() {
    let results: Vec<ReviewOutcome> = (0..5).map(|_| answer(Grade::Good)).collect();

    // Четыре по десять и пятый — по пятнадцать.
    assert_eq!(blitz_score(&results).points, 4 * BLITZ_POINTS + 15);
}

#[test]
fn ten_in_a_row_pay_double() {
    let results: Vec<ReviewOutcome> = (0..10).map(|_| answer(Grade::Good)).collect();

    // 4 × 10, потом 5 × 15 на множителе полтора, и десятый — 20.
    assert_eq!(blitz_score(&results).points, 4 * BLITZ_POINTS + 5 * 15 + 20);
    assert_eq!(blitz_score(&results).best_streak, 10);
}

#[test]
fn the_multiplier_falls_back_after_a_miss() {
    let mut results: Vec<ReviewOutcome> = (0..6).map(|_| answer(Grade::Good)).collect();
    results.push(answer(Grade::Again));
    results.push(answer(Grade::Good));

    let score = blitz_score(&results);

    // 4 × 10 + 15 + 15 за серию, ноль за промах и снова 10 с чистого листа.
    assert_eq!(score.points, 4 * BLITZ_POINTS + 15 + 15 + BLITZ_POINTS);
    assert_eq!(score.best_streak, 6);
}

#[test]
fn a_run_with_nothing_answered_scores_nothing() {
    let score = blitz_score(&[]);

    assert_eq!(score.points, 0);
    assert_eq!(score.best_streak, 0);
}
