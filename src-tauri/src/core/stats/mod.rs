//! Aggregations over study sessions and card reviews.
//!
//! Pure reducers: rows in, numbers out. Nothing here reads the clock or the
//! database — the caller passes both the records and which day «today» is.
//!
//! Split by what is being counted: [`streak`] for the days-in-a-row figure,
//! [`run`] for the numbers under one finished run of cards, [`time`] and
//! [`cards`] for the two halves of the statistics screen, [`csv`] for
//! handing any of it over as a table. The names the rest of the crate
//! already used are re-exported here, so `crate::core::stats::streak` keeps
//! working.

pub mod cards;
pub mod csv;
pub mod run;
pub mod streak;
pub mod time;

pub use run::{
    blitz_score, review_summary, BlitzScore, ReviewOutcome, ReviewSummary, BLITZ_POINTS,
    BLITZ_STREAK_DOUBLE, BLITZ_STREAK_HALF,
};
pub use streak::{
    milestones, month_days, streak, streak_state, DayMark, DayState, Milestone, StreakRules,
    StreakState, FREEZE_EVERY_DAYS, MAX_FREEZES, MILESTONES, STREAK_MIN_SECONDS,
    STREAK_WINDOW_DAYS,
};

/// `part` of `whole` as a percentage, rounded to the nearest.
///
/// Zero for an empty `whole`: nothing answered has no accuracy, and the
/// screens show a dash there rather than a nought. Integer arithmetic
/// throughout — 2 out of 3 is 67%, not 66%.
pub fn percent(part: u32, whole: u32) -> u32 {
    if whole == 0 {
        return 0;
    }

    ((u64::from(part) * 200 + u64::from(whole)) / (u64::from(whole) * 2)) as u32
}
