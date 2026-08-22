//! What can be done to a run: start it, turn a card over, grade it, and read
//! what it added up to.

use std::collections::HashMap;

use chrono::{Local, TimeDelta};
use tauri::State;

use crate::core::clock::Clock;
use crate::core::dayline::day_key;
use crate::core::review::Grade;
use crate::core::scheduler::weights::CardHistory;
use crate::core::scheduler::{weakest, CardAccuracy, StudyMode, WEAK_LIMIT, WEAK_MIN_SHOWS};
use crate::core::settings::blitz_record_key;
use crate::db::cards::CardRepo;
use crate::db::decks::DeckRepo;
use crate::db::reviews::{NewReview, ReviewRepo};
use crate::db::settings::SettingsRepo;
use crate::db::Database;
use crate::platform::clock::SystemClock;

use super::super::cards::CardView;
use super::super::settings::{adaptive_exponent, blitz_seconds, day_start};
use super::super::CommandError;
use super::plan::{histories_from_rows, order_by_weight, Plan};
use super::{begin, conflict, no_run, summarise, view, StudyState, StudySummaryView, StudyView};

/// How far back «слабые» looks. A month is long enough to have data and
/// short enough that a card learnt since then is not still called weak.
const WEAK_WINDOW_DAYS: i64 = 30;

/// How far back the picker reads a deck's answers when weighing it.
///
/// Longer than «слабые» looks, because a weight is not a verdict: half a year
/// of answers is enough for the window of recent ones to be full for every
/// card that has ever been studied, and old enough rows change nothing —
/// only the last [`crate::core::scheduler::weights::RECENT_ANSWERS`] of them
/// count towards accuracy.
const HISTORY_WINDOW_DAYS: i64 = 180;

/// What `reviews` remembers about the cards of one deck.
fn deck_histories(
    db: &Database,
    deck_id: &str,
    clock: &dyn Clock,
) -> Result<HashMap<String, CardHistory>, CommandError> {
    let since = clock.now() - TimeDelta::days(HISTORY_WINDOW_DAYS);

    Ok(histories_from_rows(
        ReviewRepo::new(db).history_for_deck(deck_id, since)?,
    ))
}

/// Picks the cards of a «слабые» run: worst accuracy first.
fn weak_cards(
    db: &Database,
    deck_id: &str,
    cards: Vec<CardView>,
    clock: &dyn Clock,
) -> Result<Vec<CardView>, CommandError> {
    let since = clock.now() - TimeDelta::days(WEAK_WINDOW_DAYS);
    let stats: Vec<CardAccuracy> = ReviewRepo::new(db)
        .accuracy_by_card(deck_id, since)?
        .into_iter()
        .map(|(card_id, shown, correct)| CardAccuracy {
            card_id,
            shown,
            correct,
        })
        .collect();

    let picked = weakest(&stats, WEAK_LIMIT, WEAK_MIN_SHOWS);
    if picked.is_empty() {
        return Err(conflict(
            "пока не по чему выбирать: карточки нужно сначала пройти хотя бы трижды",
        ));
    }

    Ok(picked
        .iter()
        .filter_map(|id| cards.iter().find(|card| &card.id == id).cloned())
        .collect())
}

/// Starts a run through a deck.
///
/// `seed` is a parameter so a run can be replayed exactly in a test; in the
/// app it comes from the clock.
pub fn start(
    db: &Database,
    state: &StudyState,
    clock: &dyn Clock,
    deck_id: &str,
    mode: StudyMode,
    seed: u64,
) -> Result<StudyView, CommandError> {
    let deck = DeckRepo::new(db)
        .get(deck_id)?
        .filter(|deck| deck.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("колода"))?;

    let cards = super::super::cards::list(db, deck_id)?;
    let histories = deck_histories(db, deck_id, clock)?;
    let exponent = adaptive_exponent(db)?;
    let now = clock.now();

    // Порядок «слабых» задаёт их точность, марафон проходит колоду целиком —
    // им остаётся решить, в каком порядке. Остальные режимы тянут карточку
    // по одной, и вес пересчитывается после каждого ответа.
    let plan = match mode {
        StudyMode::Weak => Plan::fixed(weak_cards(db, deck_id, cards, clock)?, histories, exponent),
        StudyMode::Marathon => {
            let ordered = order_by_weight(cards, &histories, exponent, now, seed);
            Plan::fixed(ordered, histories, exponent)
        }
        _ => Plan::adaptive(cards, histories, exponent, now, seed),
    };
    let seconds = mode.is_timed().then(|| blitz_seconds(db)).transpose()?;

    let run = begin(deck_id, &deck.name, mode, plan, seconds, clock)?;
    let drawn = view(&run);
    *state.lock() = Some(run);

    Ok(drawn)
}

/// The run as it stands, or `None` if there is none.
pub fn current(state: &StudyState) -> Option<StudyView> {
    state.lock().as_ref().map(view)
}

/// Turns the card over, remembering when.
pub fn reveal(state: &StudyState, clock: &dyn Clock) -> Result<StudyView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_run)?;

    if run.position >= run.queue.len() {
        return Err(conflict("прогон закончен"));
    }
    // Повторное раскрытие не сдвигает отсчёт: студент мог нажать дважды.
    if run.revealed_at.is_none() {
        run.revealed_at = Some(clock.now());
    }

    Ok(view(run))
}

/// Grades the card on screen, writes the answer down and moves on.
///
/// In a blitz a card that ran out of time is written down as «не помню»
/// whatever was pressed, and does not need to have been revealed — the clock
/// answered for the student.
pub fn answer(
    db: &Database,
    state: &StudyState,
    clock: &dyn Clock,
    grade: Grade,
) -> Result<StudyView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_run)?;

    let Some(card) = run.queue.get(run.position).cloned() else {
        return Err(conflict("прогон закончен"));
    };

    let now = clock.now();
    let late = run.is_late(now);
    if !late && run.revealed_at.is_none() {
        return Err(conflict("сначала надо посмотреть ответ"));
    }

    let grade = if late { Grade::Again } else { grade };
    let think_ms = run
        .revealed_at
        .map(|revealed| (revealed - run.shown_at).num_milliseconds().max(0));
    let total_ms = (now - run.shown_at).num_milliseconds().max(0);

    // Пишется до того, как прогон сдвинется: если запись не удалась, карточка
    // остаётся на экране и ответ можно повторить.
    ReviewRepo::new(db).create(NewReview {
        card_id: &card.id,
        reviewed_at: now,
        day_key: &day_key(now, &Local, day_start(db)?),
        result: grade.as_str(),
        correct: grade.is_correct(),
        mode: run.mode.as_str(),
        think_ms,
        total_ms: Some(total_ms),
        device_id: None,
    })?;

    // Вес карточки пересчитывается сразу: «не помню» должно вернуть её
    // в пределах ближайших нескольких карточек, а не следующего захода.
    let cache = run.plan.answered(&card.id, grade, now);
    // Кэш производный — если его не удалось записать, ответ всё равно
    // засчитан, а вес пересчитается из `reviews` в следующий раз.
    let _ = CardRepo::new(db).cache_weight(&card.id, cache);

    run.results.push(crate::core::stats::ReviewOutcome {
        card_id: card.id,
        grade,
        total_ms,
    });
    run.position += 1;
    run.shown_at = now;
    run.revealed_at = None;

    if run.position < run.total {
        match run.plan.deal() {
            Some(next) => run.queue.push(next),
            // Тянуть больше неоткуда: прогон заканчивается на том, что уже
            // показано, а не зависает на пустом экране.
            None => run.total = run.queue.len(),
        }
    }

    if run.position >= run.total {
        keep_record(db, run)?;
    }

    Ok(view(run))
}

/// Stores a blitz result if it beat what the deck had.
fn keep_record(db: &Database, run: &mut super::StudyRun) -> Result<(), CommandError> {
    if !run.mode.is_timed() {
        return Ok(());
    }

    let scored = crate::core::stats::blitz_score(&run.results).points;
    let repo = SettingsRepo::new(db);
    let key = blitz_record_key(&run.deck_id);
    let stored = repo
        .get(&key)?
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);

    if scored > stored {
        repo.set(&key, &scored.to_string())?;
        run.record_beaten = true;
    }

    Ok(())
}

/// The deck's blitz record, or `None` if it has never been played.
fn record_of(db: &Database, deck_id: &str) -> Result<Option<i64>, CommandError> {
    Ok(SettingsRepo::new(db)
        .get(&blitz_record_key(deck_id))?
        .and_then(|value| value.parse::<i64>().ok()))
}

/// The numbers under a run. Available while it is still going, too — the
/// screen after the last card and the one for a run abandoned halfway are
/// the same screen.
pub fn summary(db: &Database, state: &StudyState) -> Result<StudySummaryView, CommandError> {
    let active = state.lock();
    let run = active.as_ref().ok_or_else(no_run)?;

    let record = run
        .mode
        .is_timed()
        .then(|| record_of(db, &run.deck_id))
        .transpose()?
        .flatten();

    Ok(summarise(run, record))
}

/// Starts a new run over just the cards missed in the one that finished.
pub fn repeat_mistakes(state: &StudyState, clock: &dyn Clock) -> Result<StudyView, CommandError> {
    let mut active = state.lock();
    let run = active.as_ref().ok_or_else(no_run)?;

    let missed = crate::core::stats::review_summary(&run.results).mistakes;
    let cards: Vec<CardView> = missed
        .iter()
        .filter_map(|id| run.queue.iter().find(|card| &card.id == id))
        .cloned()
        .collect();

    if cards.is_empty() {
        return Err(conflict("ошибок в этом прогоне не было"));
    }

    // Повтор идёт в том же режиме: блиц остаётся блицем, со своим временем.
    // Порядок при этом фиксирован — это разбор конкретных ошибок, а не ещё
    // один заход по колоде.
    let next = begin(
        &run.deck_id,
        &run.deck_name,
        run.mode,
        run.plan.repeat(cards),
        run.seconds_per_card,
        clock,
    )?;
    let drawn = view(&next);
    *active = Some(next);

    Ok(drawn)
}

/// Ends the run. Answers already given stay in `reviews` — they happened.
pub fn stop(state: &StudyState) {
    *state.lock() = None;
}

/// A seed for a run that nobody asked to reproduce.
fn seed_from(clock: &dyn Clock) -> u64 {
    clock.now().timestamp_nanos_opt().unwrap_or(0) as u64
}

#[tauri::command]
pub fn study_start(
    db: State<'_, Database>,
    state: State<'_, StudyState>,
    deck_id: String,
    mode: String,
) -> Result<StudyView, CommandError> {
    start(
        &db,
        &state,
        &SystemClock,
        &deck_id,
        StudyMode::parse(&mode)?,
        seed_from(&SystemClock),
    )
}

#[tauri::command]
pub fn study_current(state: State<'_, StudyState>) -> Option<StudyView> {
    current(&state)
}

#[tauri::command]
pub fn study_reveal(state: State<'_, StudyState>) -> Result<StudyView, CommandError> {
    reveal(&state, &SystemClock)
}

#[tauri::command]
pub fn study_answer(
    db: State<'_, Database>,
    state: State<'_, StudyState>,
    grade: String,
) -> Result<StudyView, CommandError> {
    answer(&db, &state, &SystemClock, Grade::parse(&grade)?)
}

/// The blitz clock ran out. Same as answering «не помню», and refused in the
/// modes that have no clock.
#[tauri::command]
pub fn study_timeout(
    db: State<'_, Database>,
    state: State<'_, StudyState>,
) -> Result<StudyView, CommandError> {
    answer(&db, &state, &SystemClock, Grade::Again)
}

#[tauri::command]
pub fn study_summary(
    db: State<'_, Database>,
    state: State<'_, StudyState>,
) -> Result<StudySummaryView, CommandError> {
    summary(&db, &state)
}

#[tauri::command]
pub fn study_repeat_mistakes(state: State<'_, StudyState>) -> Result<StudyView, CommandError> {
    repeat_mistakes(&state, &SystemClock)
}

#[tauri::command]
pub fn study_stop(state: State<'_, StudyState>) {
    stop(&state)
}
