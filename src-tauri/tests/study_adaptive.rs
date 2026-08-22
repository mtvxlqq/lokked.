//! Tests for the adaptive pick inside a real run: what a missed card does to
//! the rest of the sitting, which cards a deck with a history deals, and what
//! is cached on the card afterwards.
//!
//! The weights themselves are tested in `adaptive.rs`, without a database.
//! Here it is the wiring: settings read, history loaded, weight recomputed
//! after every answer.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::cards::{self, CardInput};
use lokked_lib::commands::decks::{self, DeckInput};
use lokked_lib::commands::settings::write_adaptive;
use lokked_lib::commands::study::{actions as study, StudyState, StudyView};
use lokked_lib::core::clock::{Clock, FakeClock};
use lokked_lib::core::dayline::day_key;
use lokked_lib::core::review::Grade;
use lokked_lib::core::scheduler::StudyMode;
use lokked_lib::db::cards::CardRepo;
use lokked_lib::db::reviews::{NewReview, ReviewRepo};
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

/// Запускает прогон в нужном режиме с фиксированным сидом.
fn run(env: &Env, deck: &str, mode: StudyMode) -> StudyView {
    study::start(&env.db, &env.state, &env.clock, deck, mode, 1).unwrap()
}

/// Ids колоды в том порядке, в каком карточки заводились.
fn card_ids(env: &Env, deck: &str) -> Vec<String> {
    cards::list(&env.db, deck)
        .unwrap()
        .into_iter()
        .map(|card| card.id)
        .collect()
}

/// Пишет ответ прямо в `reviews`, минуя прогон: так набирается история,
/// на которую опирается подбор.
fn record(env: &Env, card_id: &str, grade: Grade, days_ago: i64) {
    let at = env.clock.now() - TimeDelta::days(days_ago);

    ReviewRepo::new(&env.db)
        .create(NewReview {
            card_id,
            reviewed_at: at,
            day_key: &day_key(at, &chrono::Local, TimeDelta::zero()),
            result: grade.as_str(),
            correct: grade.is_correct(),
            mode: "classic",
            think_ms: Some(1_000),
            total_ms: Some(2_000),
            device_id: None,
        })
        .unwrap();
}

/// Фронты карточек прогона, в порядке показа, до самого конца.
fn deal_out(env: &Env, first: StudyView) -> Vec<String> {
    let mut fronts = vec![first.card.expect("прогон начинается с карточки").front];

    loop {
        let view = answer(env, Grade::Good);
        match view.card {
            Some(card) => fronts.push(card.front),
            None => return fronts,
        }
    }
}

#[test]
fn a_card_just_missed_comes_back_inside_the_same_run() {
    // Ради этого вес и пересчитывается по ходу прогона, а не только при
    // старте: «не помню» должно вернуть карточку сейчас, а не через заход.
    let env = env();
    let deck = deck_with(&env, 20);

    let view = run(&env, &deck, StudyMode::Classic);
    let missed = view.card.unwrap().front;
    answer(&env, Grade::Again);

    let mut came_back = false;
    for _ in 0..10 {
        let on_screen = study::current(&env.state).unwrap();
        if on_screen.card.map(|card| card.front) == Some(missed.clone()) {
            came_back = true;
            break;
        }
        answer(&env, Grade::Good);
    }

    assert!(
        came_back,
        "карточка {missed} не вернулась за десять показов"
    );
}

#[test]
fn a_run_leans_towards_the_cards_going_badly() {
    let env = env();
    let deck = deck_with(&env, 40);
    let ids = card_ids(&env, &deck);

    // Пять карточек не даются, остальные идут ровно.
    for (number, id) in ids.iter().enumerate() {
        let grade = if number < 5 {
            Grade::Again
        } else {
            Grade::Good
        };
        for days_ago in 1..=4 {
            record(&env, id, grade, days_ago);
        }
    }

    let view = run(&env, &deck, StudyMode::Classic);
    let dealt = deal_out(&env, view);
    let weak: Vec<String> = (1..=5).map(|number| format!("Карточка {number}")).collect();
    let shown: Vec<&String> = weak.iter().filter(|front| dealt.contains(front)).collect();

    assert_eq!(dealt.len(), 20);
    // Простым перемешиванием все пятеро попали бы в заход примерно в трёх
    // случаях из ста: половина колоды остаётся за бортом.
    assert_eq!(
        shown.len(),
        5,
        "не все слабые карточки попали в заход: {dealt:?}"
    );
}

#[test]
fn a_marathon_still_deals_every_card_exactly_once() {
    // Вес решает, когда карточка выпадет, а не выпадет ли вообще: пройти
    // колоду целиком — это и есть марафон.
    let env = env();
    let deck = deck_with(&env, 12);

    let view = run(&env, &deck, StudyMode::Marathon);
    let mut dealt = deal_out(&env, view);
    dealt.sort();
    dealt.dedup();

    assert_eq!(dealt.len(), 12);
}

#[test]
fn answering_a_card_writes_its_weight_down() {
    // Кэш в `cards` — производная от `reviews`, но он должен поспевать за
    // ответами, а не оставаться пустым.
    let env = env();
    let deck = deck_with(&env, 5);
    let shown = run(&env, &deck, StudyMode::Classic).card.unwrap().id;

    answer(&env, Grade::Again);

    let cache = CardRepo::new(&env.db)
        .weight_cache(&shown)
        .unwrap()
        .expect("после ответа вес карточки записан");
    assert_eq!(cache.reps, 1);
    assert_eq!(cache.lapses, 1);
    assert!(cache.weight > 0.0);
}

#[test]
fn a_card_nobody_has_answered_has_no_weight_written_down() {
    let env = env();
    let deck = deck_with(&env, 3);
    let untouched = card_ids(&env, &deck).pop().unwrap();

    assert_eq!(
        CardRepo::new(&env.db).weight_cache(&untouched).unwrap(),
        None
    );
}

#[test]
fn a_flat_slider_still_deals_a_full_run() {
    // Ползунок на нуле — обычное перемешивание: подбор обязан работать и так.
    let env = env();
    let deck = deck_with(&env, 6);
    write_adaptive(&env.db, 0).unwrap();

    let view = run(&env, &deck, StudyMode::Classic);

    assert_eq!(view.total, 6);
    assert_eq!(deal_out(&env, view).len(), 6);
}
