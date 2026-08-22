//! Tests for the duel commands: how a duel is dealt, how the device is
//! passed around, what is written down, and whose answers count as study.
//!
//! The rules themselves are tested in `duel.rs` without a database. Here it
//! is the wiring: the same sequence for everyone, scores hidden until the
//! end, and a guest's answers staying out of the owner's history.

use chrono::{TimeZone, Utc};
use lokked_lib::commands::cards::{self, CardInput};
use lokked_lib::commands::decks::{self, DeckInput};
use lokked_lib::commands::duel::{actions as duel, DuelState, DuelView};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::clock::{Clock, FakeClock};
use lokked_lib::core::duel::DEFAULT_DUEL_CARDS;
use lokked_lib::core::review::Grade;
use lokked_lib::db::duels::DuelRepo;
use lokked_lib::db::reviews::ReviewRepo;
use lokked_lib::db::Database;

struct Env {
    db: Database,
    state: DuelState,
    clock: FakeClock,
}

fn env() -> Env {
    Env {
        db: Database::open_in_memory().expect("in-memory database should open"),
        state: DuelState::default(),
        clock: FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 18, 0, 0).unwrap()),
    }
}

/// A deck with `count` cards, named «Карточка N».
fn deck_with(env: &Env, count: usize) -> String {
    let deck = decks::create(
        &env.db,
        DeckInput {
            subject_id: None,
            name: "Линейная алгебра".to_string(),
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

/// Starts a duel of `names` over a deck of `cards` cards.
fn start(env: &Env, deck: &str, names: &[&str], cards: usize) -> DuelView {
    duel::start(
        &env.db,
        &env.state,
        &env.clock,
        duel::DuelStart {
            deck_id: deck,
            names: names.iter().map(|name| (*name).to_string()).collect(),
            cards,
            seconds_per_card: 20,
            seed: 2026,
        },
    )
    .unwrap()
}

/// Reveals and grades the card on screen.
fn answer(env: &Env, grade: Grade) -> DuelView {
    duel::reveal(&env.state, &env.clock).unwrap();
    duel::answer(&env.db, &env.state, &env.clock, grade).unwrap()
}

/// Plays a whole turn, answering `grade` to every card, and returns the
/// fronts of the cards in the order they came.
fn play_turn(env: &Env, grade: Grade) -> Vec<String> {
    duel::begin_turn(&env.state, &env.clock).unwrap();

    let mut fronts = Vec::new();
    loop {
        let view = duel::current(&env.state).unwrap();
        match view.card {
            Some(card) => fronts.push(card.front),
            None => return fronts,
        }
        answer(env, grade);
    }
}

// --- начало дуэли ----------------------------------------------------------

#[test]
fn a_duel_starts_on_the_hand_over_screen() {
    // Даже первый игрок начинает с «я готов»: дуэль начинается, когда
    // устройство у того, чей ход.
    let env = env();
    let deck = deck_with(&env, 30);

    let view = start(&env, &deck, &["Ты", "Артём"], 20);

    assert!(view.handover);
    assert_eq!(view.card, None);
    assert_eq!(view.current_name, "Ты");
    assert_eq!(view.turn, 1);
    assert_eq!(view.turns, 2);
    assert_eq!(view.total, 20);
    assert_eq!(view.seconds_per_card, 20);
}

#[test]
fn a_deck_too_small_for_the_duel_is_refused() {
    let env = env();
    let deck = deck_with(&env, 8);

    let error = duel::start(
        &env.db,
        &env.state,
        &env.clock,
        duel::DuelStart {
            deck_id: &deck,
            names: vec!["Ты".to_string(), "Артём".to_string()],
            cards: DEFAULT_DUEL_CARDS,
            seconds_per_card: 20,
            seed: 1,
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(error.message.contains("не хватит"), "{}", error.message);
}

#[test]
fn a_duel_of_one_player_is_not_a_duel() {
    let env = env();
    let deck = deck_with(&env, 30);

    let error = duel::start(
        &env.db,
        &env.state,
        &env.clock,
        duel::DuelStart {
            deck_id: &deck,
            names: vec!["Ты".to_string()],
            cards: 20,
            seconds_per_card: 20,
            seed: 1,
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
}

#[test]
fn the_reel_picks_a_deck_out_of_what_there_is() {
    let env = env();
    let first = deck_with(&env, 10);

    let picked = duel::pick_deck(&env.db, 7).unwrap();

    assert_eq!(picked.id, first);
}

#[test]
fn the_reel_has_nothing_to_pick_from_in_an_empty_library() {
    let env = env();

    assert_eq!(
        duel::pick_deck(&env.db, 1).unwrap_err().kind,
        ErrorKind::Conflict
    );
}

// --- ход и передача устройства ---------------------------------------------

#[test]
fn everyone_answers_the_same_cards_in_the_same_order() {
    // Ради этого дуэль и раздаёт последовательность один раз: иначе счёт
    // сравнивает не игроков, а везение.
    let env = env();
    let deck = deck_with(&env, 40);
    start(&env, &deck, &["Ты", "Артём"], 10);

    let first = play_turn(&env, Grade::Good);
    let second = play_turn(&env, Grade::Again);

    assert_eq!(first.len(), 10);
    assert_eq!(first, second);
}

#[test]
fn the_device_is_handed_over_between_turns() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Good);

    let view = duel::current(&env.state).unwrap();

    assert!(view.handover, "перед ходом гостя — экран передачи");
    assert_eq!(view.current_name, "Артём");
    assert_eq!(view.turn, 2);
    assert_eq!(view.card, None, "карточка не показана, пока не готов");
    // Свой счёт у нового игрока пустой, чужой не показывается вовсе.
    assert_eq!(view.points, 0);
    assert!(view.players[0].played);
    assert!(!view.players[1].played);
}

#[test]
fn a_card_cannot_be_answered_before_the_player_says_they_are_ready() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);

    assert_eq!(
        duel::reveal(&env.state, &env.clock).unwrap_err().kind,
        ErrorKind::Conflict
    );
    assert_eq!(
        duel::answer(&env.db, &env.state, &env.clock, Grade::Good)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
}

#[test]
fn the_answer_has_to_be_looked_at_first() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    duel::begin_turn(&env.state, &env.clock).unwrap();

    assert_eq!(
        duel::answer(&env.db, &env.state, &env.clock, Grade::Good)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
}

#[test]
fn a_card_that_ran_out_of_time_counts_as_not_remembered() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    duel::begin_turn(&env.state, &env.clock).unwrap();

    // Двадцать одна секунда на карточке, которой дано двадцать.
    env.clock.advance(chrono::TimeDelta::seconds(21));
    duel::answer(&env.db, &env.state, &env.clock, Grade::Easy).unwrap();

    let answers = DuelRepo::new(&env.db)
        .answers(&duel::current(&env.state).unwrap().duel_id)
        .unwrap();
    assert_eq!(answers[0].result, "again");
    assert!(!answers[0].correct);
}

#[test]
fn the_clock_starts_when_the_reel_stops() {
    // Полтора секунды прокрута не должны съедать время карточки: часы
    // заводятся, когда на карточку уже можно смотреть.
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    let dealt = duel::begin_turn(&env.state, &env.clock).unwrap();

    env.clock.advance(chrono::TimeDelta::seconds(2));
    let spun = duel::settled(&env.state, &env.clock).unwrap();

    assert!(
        spun.deadline > dealt.deadline,
        "дедлайн обязан сдвинуться вместе с барабаном"
    );
}

#[test]
fn the_clock_is_not_extended_by_a_second_report_from_the_reel() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    duel::begin_turn(&env.state, &env.clock).unwrap();
    duel::settled(&env.state, &env.clock).unwrap();
    duel::reveal(&env.state, &env.clock).unwrap();

    let before = duel::current(&env.state).unwrap().deadline;
    env.clock.advance(chrono::TimeDelta::seconds(3));
    let after = duel::settled(&env.state, &env.clock).unwrap().deadline;

    assert_eq!(after, before, "после раскрытия часы уже не переставить");
}

#[test]
fn the_duel_ends_after_the_last_turn() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Good);
    play_turn(&env, Grade::Good);

    let view = duel::current(&env.state).unwrap();

    assert!(view.finished);
    assert!(!view.handover);
    assert_eq!(view.card, None);
}

#[test]
fn four_players_take_four_turns() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём", "Соня", "Илья"], 5);

    for turn in 1..=4 {
        assert_eq!(duel::current(&env.state).unwrap().turn, turn);
        play_turn(&env, Grade::Good);
    }

    assert!(duel::current(&env.state).unwrap().finished);
}

// --- что записано ----------------------------------------------------------

#[test]
fn a_guests_answers_stay_out_of_the_owners_history() {
    // Главное правило дуэли: вечер в гостях не должен переписывать чужую
    // статистику и чужие веса карточек.
    let env = env();
    let deck = deck_with(&env, 30);
    let view = start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Good);
    play_turn(&env, Grade::Again);

    let reviews = ReviewRepo::new(&env.db)
        .list_for_day(&lokked_lib::core::dayline::day_key(
            env.clock.now(),
            &chrono::Local,
            chrono::TimeDelta::zero(),
        ))
        .unwrap();
    assert_eq!(reviews.len(), 5, "в личную историю попал только свой ход");
    assert!(reviews.iter().all(|review| review.mode == "duel"));
    assert!(reviews.iter().all(|review| review.correct));

    // При этом в самой дуэли записаны оба хода целиком.
    let answers = DuelRepo::new(&env.db).answers(&view.duel_id).unwrap();
    assert_eq!(answers.len(), 10);
}

#[test]
fn the_duel_its_players_and_its_scores_are_written_down() {
    let env = env();
    let deck = deck_with(&env, 30);
    let view = start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Good);
    play_turn(&env, Grade::Again);

    let repo = DuelRepo::new(&env.db);
    let stored = repo.get(&view.duel_id).unwrap().expect("дуэль записана");
    assert_eq!(stored.deck_id, deck);
    assert_eq!(stored.cards, 5);
    assert!(stored.finished_at.is_some(), "дуэль закрыта");

    let players = repo.players(&view.duel_id).unwrap();
    assert_eq!(players.len(), 2);
    assert!(players[0].is_owner);
    assert!(!players[1].is_owner);
    assert!(players[0].points > 0, "победитель что-то набрал");
    assert_eq!(players[1].points, 0, "гость не ответил ни одной");
    assert_eq!(players[0].correct, 5);
}

#[test]
fn a_duel_left_halfway_keeps_what_was_answered() {
    let env = env();
    let deck = deck_with(&env, 30);
    let view = start(&env, &deck, &["Ты", "Артём"], 5);
    duel::begin_turn(&env.state, &env.clock).unwrap();
    answer(&env, Grade::Good);
    answer(&env, Grade::Again);
    duel::stop(&env.state);

    let repo = DuelRepo::new(&env.db);
    assert_eq!(repo.answers(&view.duel_id).unwrap().len(), 2);
    assert!(
        repo.get(&view.duel_id)
            .unwrap()
            .unwrap()
            .finished_at
            .is_none(),
        "брошенная дуэль остаётся незакрытой"
    );
    assert!(duel::current(&env.state).is_none());
}

// --- итоги -----------------------------------------------------------------

#[test]
fn the_summary_names_the_winner_and_breaks_the_duel_down_by_card() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Good);
    play_turn(&env, Grade::Again);

    let summary = duel::summary(&env.state).unwrap();

    assert_eq!(summary.players.len(), 2);
    assert!(summary.players[0].winner);
    assert!(!summary.players[1].winner);
    assert_eq!(summary.players[0].correct, 5);
    assert_eq!(summary.players[1].correct, 0);
    assert!(summary.players[0].points > summary.players[1].points);

    assert_eq!(summary.breakdown.len(), 5);
    assert_eq!(
        summary.breakdown[0].answers,
        vec![Some("good".to_string()), Some("again".to_string())]
    );
    // Оборот в разборе виден: дуэль закончена, скрывать больше нечего.
    assert!(!summary.breakdown[0].back.is_empty());
}

#[test]
fn an_equal_duel_is_a_draw() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Good);
    play_turn(&env, Grade::Good);

    let summary = duel::summary(&env.state).unwrap();

    assert!(summary.players.iter().all(|player| player.winner));
    assert_eq!(summary.players[0].points, summary.players[1].points);
}

#[test]
fn a_duel_nobody_answered_right_has_no_winner() {
    let env = env();
    let deck = deck_with(&env, 30);
    start(&env, &deck, &["Ты", "Артём"], 5);
    play_turn(&env, Grade::Again);
    play_turn(&env, Grade::Again);

    let summary = duel::summary(&env.state).unwrap();

    assert!(summary.players.iter().all(|player| !player.winner));
}

#[test]
fn without_a_duel_there_is_nothing_to_show() {
    let env = env();

    assert!(duel::current(&env.state).is_none());
    assert_eq!(
        duel::summary(&env.state).unwrap_err().kind,
        ErrorKind::Conflict
    );
    assert_eq!(
        duel::begin_turn(&env.state, &env.clock).unwrap_err().kind,
        ErrorKind::Conflict
    );
}
