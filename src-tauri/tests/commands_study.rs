//! Tests for a run through a deck: what the screen is given, what lands in
//! `reviews`, and what the run adds up to.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::cards::{self, CardInput};
use lokked_lib::commands::decks::{self, DeckInput};
use lokked_lib::commands::study::{actions as study, StudyState, StudyView};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::clock::{Clock, FakeClock};
use lokked_lib::core::review::Grade;
use lokked_lib::core::scheduler::StudyMode;
use lokked_lib::db::reviews::ReviewRepo;
use lokked_lib::db::Database;

/// Everything a study command needs, wired to fakes.
struct Env {
    db: Database,
    state: StudyState,
    clock: FakeClock,
}

fn env() -> Env {
    Env {
        db: Database::open_in_memory().expect("in-memory database should open"),
        state: StudyState::default(),
        clock: FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 18, 0, 0).unwrap()),
    }
}

/// A deck with `count` cards, named «Карточка N».
fn deck_with(env: &Env, count: usize) -> String {
    let deck = decks::create(
        &env.db,
        DeckInput {
            subject_id: None,
            name: "Матанализ".to_string(),
            description: None,
        },
    )
    .unwrap();

    for number in 1..=count {
        cards::create(
            &env.db,
            &deck.id,
            CardInput {
                front: format!("Карточка {number}"),
                back: format!("Оборот {number}"),
                hint: None,
                tags: Vec::new(),
            },
        )
        .unwrap();
    }

    deck.id
}

/// Reveals and grades the card on screen.
fn answer(env: &Env, grade: Grade) -> StudyView {
    study::reveal(&env.state, &env.clock).unwrap();
    study::answer(&env.db, &env.state, &env.clock, grade).unwrap()
}

fn reviews(env: &Env) -> Vec<lokked_lib::db::reviews::Review> {
    let day =
        lokked_lib::core::dayline::day_key(env.clock.now(), &chrono::Local, TimeDelta::zero());
    ReviewRepo::new(&env.db).list_for_day(&day).unwrap()
}

// --- начало прогона --------------------------------------------------------

#[test]
fn a_run_starts_on_the_first_card_with_the_answer_hidden() {
    let env = env();
    let deck = deck_with(&env, 3);

    let view = study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    assert_eq!(view.total, 3);
    assert_eq!(view.position, 1);
    assert_eq!(view.answered, 0);
    assert!(!view.revealed);
    assert!(!view.finished);
    // Оборот не уезжает на экран, пока его не попросили.
    assert_eq!(view.card.unwrap().back, None);
}

#[test]
fn the_same_seed_deals_the_same_order() {
    let first = env();
    let second = env();
    let one = deck_with(&first, 12);
    let two = deck_with(&second, 12);

    let left = study::start(
        &first.db,
        &first.state,
        &first.clock,
        &one,
        StudyMode::Classic,
        2026,
    )
    .unwrap();
    let right = study::start(
        &second.db,
        &second.state,
        &second.clock,
        &two,
        StudyMode::Classic,
        2026,
    )
    .unwrap();

    assert_eq!(
        left.card.unwrap().front,
        right.card.unwrap().front,
        "с одним и тем же сидом порядок обязан совпасть"
    );
}

#[test]
fn a_deck_with_nothing_in_it_cannot_be_studied() {
    let env = env();
    let deck = deck_with(&env, 0);

    let error = study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Conflict);
    assert!(study::current(&env.state).is_none());
}

#[test]
fn a_deck_that_is_gone_cannot_be_studied() {
    let env = env();

    let error = study::start(
        &env.db,
        &env.state,
        &env.clock,
        "нет такой",
        StudyMode::Classic,
        1,
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn without_a_run_there_is_nothing_to_show_reveal_or_answer() {
    let env = env();

    assert!(study::current(&env.state).is_none());
    assert_eq!(
        study::reveal(&env.state, &env.clock).unwrap_err().kind,
        ErrorKind::Conflict
    );
    assert_eq!(
        study::answer(&env.db, &env.state, &env.clock, Grade::Good)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
}

// --- ход прогона -----------------------------------------------------------

#[test]
fn revealing_shows_the_answer() {
    let env = env();
    let deck = deck_with(&env, 2);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    let view = study::reveal(&env.state, &env.clock).unwrap();

    assert!(view.revealed);
    assert!(view.card.unwrap().back.is_some());
}

#[test]
fn grading_without_looking_at_the_answer_is_refused() {
    let env = env();
    let deck = deck_with(&env, 2);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    let error = study::answer(&env.db, &env.state, &env.clock, Grade::Good).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Conflict);
    assert!(reviews(&env).is_empty());
}

#[test]
fn revealing_twice_does_not_restart_the_stopwatch() {
    let env = env();
    let deck = deck_with(&env, 1);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    env.clock.advance(TimeDelta::seconds(4));
    study::reveal(&env.state, &env.clock).unwrap();
    env.clock.advance(TimeDelta::seconds(6));
    study::reveal(&env.state, &env.clock).unwrap();
    study::answer(&env.db, &env.state, &env.clock, Grade::Good).unwrap();

    assert_eq!(reviews(&env)[0].think_ms, Some(4_000));
}

#[test]
fn an_answer_moves_to_the_next_card_with_the_answer_hidden_again() {
    let env = env();
    let deck = deck_with(&env, 3);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    let view = answer(&env, Grade::Good);

    assert_eq!(view.position, 2);
    assert_eq!(view.answered, 1);
    assert!(!view.revealed);
    assert_eq!(view.card.unwrap().back, None);
}

#[test]
fn the_run_ends_after_the_last_card() {
    let env = env();
    let deck = deck_with(&env, 2);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    answer(&env, Grade::Good);
    let view = answer(&env, Grade::Again);

    assert!(view.finished);
    assert_eq!(view.answered, 2);
    assert!(view.card.is_none());
    assert_eq!(
        study::reveal(&env.state, &env.clock).unwrap_err().kind,
        ErrorKind::Conflict
    );
}

// --- что попадает в reviews ------------------------------------------------

#[test]
fn every_answer_is_written_down_with_its_timings() {
    let env = env();
    let deck = deck_with(&env, 1);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    env.clock.advance(TimeDelta::seconds(7));
    study::reveal(&env.state, &env.clock).unwrap();
    env.clock.advance(TimeDelta::seconds(3));
    study::answer(&env.db, &env.state, &env.clock, Grade::Hard).unwrap();

    let stored = reviews(&env);
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].result, "hard");
    assert!(stored[0].correct);
    assert_eq!(stored[0].mode, "classic");
    // Думал семь секунд, всего десять.
    assert_eq!(stored[0].think_ms, Some(7_000));
    assert_eq!(stored[0].total_ms, Some(10_000));
}

#[test]
fn not_remembering_is_written_down_as_not_correct() {
    let env = env();
    let deck = deck_with(&env, 1);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    answer(&env, Grade::Again);

    assert!(!reviews(&env)[0].correct);
}

#[test]
fn the_timings_of_the_second_card_start_from_the_second_card() {
    let env = env();
    let deck = deck_with(&env, 2);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    env.clock.advance(TimeDelta::seconds(30));
    answer(&env, Grade::Good);
    env.clock.advance(TimeDelta::seconds(2));
    study::reveal(&env.state, &env.clock).unwrap();
    study::answer(&env.db, &env.state, &env.clock, Grade::Good).unwrap();

    let stored = reviews(&env);
    assert_eq!(stored[0].total_ms, Some(30_000));
    assert_eq!(stored[1].total_ms, Some(2_000));
}

// --- итоги -----------------------------------------------------------------

#[test]
fn the_summary_counts_what_was_recalled() {
    let env = env();
    let deck = deck_with(&env, 4);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    env.clock.advance(TimeDelta::seconds(2));
    answer(&env, Grade::Good);
    env.clock.advance(TimeDelta::seconds(2));
    answer(&env, Grade::Again);
    env.clock.advance(TimeDelta::seconds(2));
    answer(&env, Grade::Easy);
    env.clock.advance(TimeDelta::seconds(2));
    answer(&env, Grade::Again);

    let summary = study::summary(&env.db, &env.state).unwrap();

    assert_eq!(summary.summary.answered, 4);
    assert_eq!(summary.summary.correct, 2);
    assert_eq!(summary.summary.accuracy_percent, 50);
    assert_eq!(summary.mistake_cards.len(), 2);
    // Ошибки показываются целиком, с ответом: их разбирают сразу.
    assert!(summary.mistake_cards[0].back.is_some());
    assert_eq!(summary.deck_name, "Матанализ");
}

#[test]
fn the_summary_of_a_run_left_halfway_counts_only_what_was_answered() {
    let env = env();
    let deck = deck_with(&env, 5);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    answer(&env, Grade::Good);

    let summary = study::summary(&env.db, &env.state).unwrap();

    assert_eq!(summary.summary.answered, 1);
    assert_eq!(summary.summary.accuracy_percent, 100);
}

#[test]
fn repeating_mistakes_deals_exactly_the_missed_cards() {
    let env = env();
    let deck = deck_with(&env, 4);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    answer(&env, Grade::Again);
    answer(&env, Grade::Good);
    answer(&env, Grade::Again);
    answer(&env, Grade::Good);

    let mut missed: Vec<String> = study::summary(&env.db, &env.state)
        .unwrap()
        .mistake_cards
        .into_iter()
        .map(|card| card.front)
        .collect();
    missed.sort();

    let mut view = study::repeat_mistakes(&env.state, &env.clock, 7).unwrap();
    assert_eq!(view.total, 2);
    assert_eq!(view.answered, 0);

    // Проходим повтор целиком и смотрим, что в нём были ровно те карточки.
    let mut dealt = Vec::new();
    while let Some(card) = view.card.clone() {
        dealt.push(card.front);
        view = answer(&env, Grade::Good);
    }
    dealt.sort();

    assert_eq!(dealt, missed);
    assert!(view.finished);
}

#[test]
fn there_is_nothing_to_repeat_after_a_clean_run() {
    let env = env();
    let deck = deck_with(&env, 2);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();

    answer(&env, Grade::Good);
    answer(&env, Grade::Easy);

    assert_eq!(
        study::repeat_mistakes(&env.state, &env.clock, 7)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
}

#[test]
fn stopping_a_run_leaves_the_answers_already_given() {
    let env = env();
    let deck = deck_with(&env, 3);
    study::start(
        &env.db,
        &env.state,
        &env.clock,
        &deck,
        StudyMode::Classic,
        1,
    )
    .unwrap();
    answer(&env, Grade::Good);

    study::stop(&env.state);

    assert!(study::current(&env.state).is_none());
    // Ответ уже случился — в истории он остаётся.
    assert_eq!(reviews(&env).len(), 1);
}

// --- режимы ----------------------------------------------------------------

/// Запускает прогон в нужном режиме с фиксированным сидом.
fn run(env: &Env, deck: &str, mode: StudyMode) -> StudyView {
    study::start(&env.db, &env.state, &env.clock, deck, mode, 1).unwrap()
}

#[test]
fn a_classic_run_is_a_sitting_not_the_whole_deck() {
    let env = env();
    let deck = deck_with(&env, 40);

    let view = run(&env, &deck, StudyMode::Classic);

    assert_eq!(view.total, 20);
    assert_eq!(view.mode, "classic");
}

#[test]
fn a_marathon_takes_everything_there_is() {
    let env = env();
    let deck = deck_with(&env, 40);

    let view = run(&env, &deck, StudyMode::Marathon);

    assert_eq!(view.total, 40);
    assert_eq!(view.mode, "marathon");
    // Часов у марафона нет.
    assert_eq!(view.deadline, None);
    assert_eq!(view.points, None);
}

#[test]
fn a_deck_smaller_than_the_sitting_is_dealt_whole() {
    let env = env();
    let deck = deck_with(&env, 3);

    assert_eq!(run(&env, &deck, StudyMode::Classic).total, 3);
}

#[test]
fn a_blitz_card_comes_with_a_deadline_and_a_score() {
    let env = env();
    let deck = deck_with(&env, 5);

    let view = run(&env, &deck, StudyMode::Blitz);

    assert_eq!(view.mode, "blitz");
    assert_eq!(view.seconds_per_card, Some(20));
    assert_eq!(
        view.deadline,
        Some(env.clock.now() + TimeDelta::seconds(20))
    );
    assert_eq!(view.points, Some(0));
    assert_eq!(view.streak, Some(0));
}

#[test]
fn the_blitz_deadline_moves_with_every_card() {
    let env = env();
    let deck = deck_with(&env, 3);
    run(&env, &deck, StudyMode::Blitz);

    env.clock.advance(TimeDelta::seconds(4));
    let view = answer(&env, Grade::Good);

    assert_eq!(
        view.deadline,
        Some(env.clock.now() + TimeDelta::seconds(20))
    );
    assert_eq!(view.points, Some(10));
    assert_eq!(view.streak, Some(1));
}

#[test]
fn a_blitz_card_that_ran_out_of_time_counts_as_not_remembered() {
    let env = env();
    let deck = deck_with(&env, 2);
    run(&env, &deck, StudyMode::Blitz);

    // Двадцать секунд прошли, ответа не было.
    env.clock.advance(TimeDelta::seconds(21));
    let view = study::answer(&env.db, &env.state, &env.clock, Grade::Easy).unwrap();

    // Нажатое не считается: за студента ответили часы.
    assert_eq!(reviews(&env)[0].result, "again");
    assert!(!reviews(&env)[0].correct);
    assert_eq!(view.points, Some(0));
    // И раскрывать ответ для этого не требовалось.
    assert_eq!(reviews(&env)[0].think_ms, None);
}

#[test]
fn time_running_out_is_only_a_thing_in_blitz() {
    let env = env();
    let deck = deck_with(&env, 2);
    run(&env, &deck, StudyMode::Classic);

    env.clock.advance(TimeDelta::minutes(5));
    let view = answer(&env, Grade::Good);

    assert_eq!(reviews(&env)[0].result, "good");
    assert_eq!(view.deadline, None);
}

#[test]
fn a_blitz_run_is_written_down_as_a_blitz() {
    let env = env();
    let deck = deck_with(&env, 1);
    run(&env, &deck, StudyMode::Blitz);

    answer(&env, Grade::Good);

    assert_eq!(reviews(&env)[0].mode, "blitz");
}

#[test]
fn the_blitz_summary_carries_the_score_and_the_record() {
    let env = env();
    let deck = deck_with(&env, 3);
    run(&env, &deck, StudyMode::Blitz);

    answer(&env, Grade::Good);
    answer(&env, Grade::Good);
    answer(&env, Grade::Again);

    let summary = study::summary(&env.db, &env.state).unwrap();

    assert_eq!(summary.points, Some(20));
    assert_eq!(summary.best_streak, Some(2));
    assert_eq!(summary.record, Some(20));
    assert!(summary.record_beaten);
}

#[test]
fn a_weaker_blitz_leaves_the_record_alone() {
    let env = env();
    let deck = deck_with(&env, 3);

    run(&env, &deck, StudyMode::Blitz);
    answer(&env, Grade::Good);
    answer(&env, Grade::Good);
    answer(&env, Grade::Good);

    run(&env, &deck, StudyMode::Blitz);
    answer(&env, Grade::Again);
    answer(&env, Grade::Again);
    answer(&env, Grade::Again);

    let summary = study::summary(&env.db, &env.state).unwrap();

    assert_eq!(summary.points, Some(0));
    assert_eq!(summary.record, Some(30));
    assert!(!summary.record_beaten);
}

#[test]
fn a_summary_of_a_run_without_a_clock_has_no_score() {
    let env = env();
    let deck = deck_with(&env, 2);
    run(&env, &deck, StudyMode::Classic);
    answer(&env, Grade::Good);

    let summary = study::summary(&env.db, &env.state).unwrap();

    assert_eq!(summary.points, None);
    assert_eq!(summary.record, None);
}

#[test]
fn the_weak_mode_deals_the_cards_with_the_worst_accuracy() {
    let env = env();
    let deck = deck_with(&env, 4);

    // Проходим колоду трижды: первая карточка каждый раз мимо, остальные — да.
    for _ in 0..3 {
        run(&env, &deck, StudyMode::Marathon);
        let mut worst = String::new();
        for index in 0..4 {
            let view = study::current(&env.state).unwrap();
            let front = view.card.unwrap().front;
            if index == 0 {
                worst = front.clone();
            }
            answer(
                &env,
                if front == worst {
                    Grade::Again
                } else {
                    Grade::Good
                },
            );
        }
    }

    let view = run(&env, &deck, StudyMode::Weak);

    assert_eq!(view.mode, "weak");
    // Первой идёт та, что валилась чаще всех.
    assert!(view.total >= 1);
    assert_eq!(view.position, 1);
}

#[test]
fn the_weak_mode_needs_a_history_to_work_from() {
    let env = env();
    let deck = deck_with(&env, 5);

    let error =
        study::start(&env.db, &env.state, &env.clock, &deck, StudyMode::Weak, 1).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Conflict);
    assert!(error.message.contains("трижды"));
}

#[test]
fn repeating_the_mistakes_of_a_blitz_is_still_a_blitz() {
    let env = env();
    let deck = deck_with(&env, 2);
    run(&env, &deck, StudyMode::Blitz);

    answer(&env, Grade::Again);
    answer(&env, Grade::Good);

    let view = study::repeat_mistakes(&env.state, &env.clock, 3).unwrap();

    assert_eq!(view.mode, "blitz");
    assert_eq!(view.total, 1);
    assert!(view.deadline.is_some());
}
