//! The «Карточки» and «Карточка» tabs: how answering went over a period,
//! which cards keep being missed, and one card's whole history.

use serde::Serialize;
use tauri::State;

use crate::core::review::Grade;
use crate::core::scheduler::{CardAccuracy, WEAK_MIN_SHOWS};
use crate::core::stats::cards::{
    accuracy_by_day, card_stats, problem_cards, CardAnswer, CardStats, DayAccuracy, ProblemCard,
};
use crate::core::stats::percent;
use crate::core::stats::time::StatsRange;
use crate::db::cards::CardRepo;
use crate::db::reviews::ReviewRepo;
use crate::db::Database;

use super::{period, today, CommandError, Period};

/// How many problem cards the tab lists.
pub const PROBLEM_LIMIT: usize = 20;

/// A weak card with enough of itself to be recognised in the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemCardView {
    #[serde(flatten)]
    pub card: ProblemCard,
    /// The front, as written — the frontend renders the markup.
    pub front: String,
    pub deck_id: String,
}

/// Everything the «Карточки» tab draws.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardsStats {
    #[serde(flatten)]
    pub period: Period,
    pub answered: u32,
    pub correct: u32,
    pub accuracy_percent: u32,
    /// A point per day of the period, days without answers included.
    pub by_day: Vec<DayAccuracy>,
    /// The cards worth going back to, worst first.
    pub problems: Vec<ProblemCardView>,
}

/// The «Карточки» tab for `range`, as of the study day `today`.
pub fn cards_stats(
    db: &Database,
    range: StatsRange,
    today: &str,
) -> Result<CardsStats, CommandError> {
    let period = period(db, range, today)?;
    let repo = ReviewRepo::new(db);

    let counts = repo.counts_by_day(&period.from, &period.to)?;
    let answered: u32 = counts.iter().map(|(_, answered, _)| answered).sum();
    let correct: u32 = counts.iter().map(|(_, _, correct)| correct).sum();

    let accuracy: Vec<CardAccuracy> = repo
        .accuracy_by_card_in_days(&period.from, &period.to)?
        .into_iter()
        .map(|(card_id, shown, correct)| CardAccuracy {
            card_id,
            shown,
            correct,
        })
        .collect();

    Ok(CardsStats {
        answered,
        correct,
        accuracy_percent: percent(correct, answered),
        by_day: accuracy_by_day(&counts, &period.from, &period.to),
        problems: with_fronts(db, problem_cards(&accuracy, PROBLEM_LIMIT, WEAK_MIN_SHOWS))?,
        period,
    })
}

/// Pairs each weak card with its front, keeping the order it came in.
fn with_fronts(
    db: &Database,
    problems: Vec<ProblemCard>,
) -> Result<Vec<ProblemCardView>, CommandError> {
    let ids: Vec<String> = problems.iter().map(|card| card.card_id.clone()).collect();
    let found = CardRepo::new(db).list_by_ids(&ids)?;

    Ok(problems
        .into_iter()
        .filter_map(|problem| {
            let card = found.iter().find(|card| card.id == problem.card_id)?;

            Some(ProblemCardView {
                front: card.front.clone(),
                deck_id: card.deck_id.clone(),
                card: problem,
            })
        })
        .collect())
}

/// One card and everything its answers add up to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardReport {
    pub card_id: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
    #[serde(flatten)]
    pub stats: CardStats,
}

/// The «Карточка» tab for one card.
pub fn card_report(db: &Database, card_id: &str) -> Result<CardReport, CommandError> {
    let card = CardRepo::new(db)
        .get(card_id)?
        .filter(|card| card.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("карточка"))?;

    let answers: Vec<CardAnswer> = ReviewRepo::new(db)
        .list_for_card(card_id)?
        .into_iter()
        .map(|review| CardAnswer {
            // Оценка из будущей версии не должна ронять экран: считаем её
            // ответом «не помню» — это осторожнее, чем зачесть незнакомое
            // за успех.
            grade: Grade::parse(&review.result).unwrap_or(Grade::Again),
            think_ms: review.think_ms,
        })
        .collect();

    Ok(CardReport {
        card_id: card.id,
        deck_id: card.deck_id,
        front: card.front,
        back: card.back,
        stats: card_stats(&answers),
    })
}

#[tauri::command]
pub fn stats_cards(db: State<'_, Database>, range: String) -> Result<CardsStats, CommandError> {
    let range = StatsRange::parse(&range)?;
    let today = today(&db)?;

    cards_stats(&db, range, &today)
}

#[tauri::command]
pub fn stats_card(db: State<'_, Database>, card_id: String) -> Result<CardReport, CommandError> {
    card_report(&db, &card_id)
}
