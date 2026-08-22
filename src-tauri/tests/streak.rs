//! Tests for the streak: what keeps it alive, what ends it, what a freeze
//! costs, and how the days are marked for the calendar.

use lokked_lib::core::stats::streak::{
    milestones, month_days, streak_state, DayState, StreakRules, FREEZE_EVERY_DAYS, MAX_FREEZES,
    MILESTONES, STREAK_MIN_SECONDS,
};

/// A day that counts, and one that does not quite.
const ENOUGH: i64 = STREAK_MIN_SECONDS;
const TOO_LITTLE: i64 = STREAK_MIN_SECONDS - 1;

/// `(day_key, active_seconds)` pairs, as they come out of the database.
fn days(pairs: &[(&str, i64)]) -> Vec<(String, i64)> {
    pairs
        .iter()
        .map(|(day, seconds)| ((*day).to_string(), *seconds))
        .collect()
}

/// `count` days ending on `last`, every one of them studied.
fn run_up_to(last: &str, count: i64) -> Vec<(String, i64)> {
    let last = chrono::NaiveDate::parse_from_str(last, "%Y-%m-%d").unwrap();

    (0..count)
        .map(|back| {
            (
                (last - chrono::TimeDelta::days(back))
                    .format("%Y-%m-%d")
                    .to_string(),
                ENOUGH,
            )
        })
        .collect()
}

fn state(recorded: &[(String, i64)], today: &str) -> lokked_lib::core::stats::streak::StreakState {
    streak_state(recorded, today, StreakRules::default())
}

/// What one day of the run was marked as.
fn mark(state: &lokked_lib::core::stats::streak::StreakState, day: &str) -> Option<DayState> {
    state
        .days
        .iter()
        .find(|marked| marked.day == day)
        .map(|marked| marked.state)
}

// --- сама серия ------------------------------------------------------------

#[test]
fn a_student_who_has_never_studied_has_nothing() {
    let empty = state(&days(&[]), "2026-08-21");

    assert_eq!(empty.current, 0);
    assert_eq!(empty.longest, 0);
    assert_eq!(empty.freezes, 0);
    assert_eq!(empty.longest_from, None);
}

#[test]
fn consecutive_days_add_up() {
    assert_eq!(state(&run_up_to("2026-08-21", 3), "2026-08-21").current, 3);
}

#[test]
fn a_missed_day_ends_a_streak_too_short_to_have_earned_a_freeze() {
    let recorded = days(&[
        ("2026-08-17", ENOUGH),
        ("2026-08-18", ENOUGH),
        // 19 августа пропущено, заморозок ещё не накоплено.
        ("2026-08-20", ENOUGH),
        ("2026-08-21", ENOUGH),
    ]);

    assert_eq!(state(&recorded, "2026-08-21").current, 2);
}

#[test]
fn a_day_that_has_only_just_begun_breaks_nothing() {
    // Серия не обнуляется в полночь: пока сегодня пусто, показывается
    // вчерашняя, а сам день ждёт своих десяти минут.
    let recorded = run_up_to("2026-08-20", 2);
    let today = state(&recorded, "2026-08-21");

    assert_eq!(today.current, 2);
    assert_eq!(mark(&today, "2026-08-21"), Some(DayState::Pending));
}

#[test]
fn a_day_below_the_minimum_does_not_count() {
    let recorded = days(&[("2026-08-20", ENOUGH), ("2026-08-21", TOO_LITTLE)]);

    assert_eq!(state(&recorded, "2026-08-21").current, 1);
}

#[test]
fn the_minimum_is_a_setting_not_a_law() {
    let recorded = days(&[("2026-08-20", 20 * 60), ("2026-08-21", 20 * 60)]);
    let rules = StreakRules {
        min_seconds: 30 * 60,
        ..StreakRules::default()
    };

    assert_eq!(streak_state(&recorded, "2026-08-21", rules).current, 0);
    assert_eq!(state(&recorded, "2026-08-21").current, 2);
}

#[test]
fn several_sessions_in_one_day_are_summed_before_the_minimum_applies() {
    let recorded = days(&[("2026-08-21", 400), ("2026-08-21", 300)]);

    assert_eq!(state(&recorded, "2026-08-21").current, 1);
}

#[test]
fn a_streak_crosses_the_turn_of_the_year() {
    let recorded = run_up_to("2027-01-05", 12);

    assert_eq!(state(&recorded, "2027-01-05").current, 12);
}

// --- заморозки -------------------------------------------------------------

#[test]
fn ten_days_in_a_row_earn_one_freeze() {
    let earned = state(
        &run_up_to("2026-08-21", FREEZE_EVERY_DAYS as i64),
        "2026-08-21",
    );

    assert_eq!(earned.current, 10);
    assert_eq!(earned.freezes, 1);
}

#[test]
fn freezes_stop_accruing_at_three() {
    let long = state(&run_up_to("2026-08-21", 40), "2026-08-21");

    assert_eq!(long.freezes, MAX_FREEZES);
}

#[test]
fn a_freeze_carries_the_streak_over_a_missed_day() {
    let mut recorded = run_up_to("2026-08-20", 10);
    // 21 августа пропущено, 22-го занятия снова.
    recorded.push(("2026-08-22".to_string(), ENOUGH));

    let carried = state(&recorded, "2026-08-22");

    assert_eq!(carried.current, 11, "серия обязана пережить пропуск");
    assert_eq!(carried.freezes, 0, "заморозка потрачена");
    assert_eq!(carried.frozen_days, 1);
    assert_eq!(mark(&carried, "2026-08-21"), Some(DayState::Frozen));
}

#[test]
fn a_frozen_day_does_not_lengthen_the_streak() {
    let mut recorded = run_up_to("2026-08-20", 10);
    recorded.push(("2026-08-22".to_string(), ENOUGH));

    // Одиннадцать занятых дней за двенадцать календарных.
    assert_eq!(state(&recorded, "2026-08-22").current, 11);
}

#[test]
fn two_missed_days_in_a_row_cost_two_freezes() {
    let mut recorded = run_up_to("2026-08-10", 20);
    // 11 и 12 августа пропущены, 13-го снова занятия.
    recorded.push(("2026-08-13".to_string(), ENOUGH));

    let carried = state(&recorded, "2026-08-13");

    assert_eq!(carried.current, 21);
    assert_eq!(carried.frozen_days, 2);
    assert_eq!(carried.freezes, 0);
}

#[test]
fn a_missed_day_with_nothing_left_to_spend_ends_the_streak() {
    let mut recorded = run_up_to("2026-08-10", 20);
    // Три пропуска подряд при двух заморозках в запасе.
    recorded.push(("2026-08-14".to_string(), ENOUGH));

    let broken = state(&recorded, "2026-08-14");

    assert_eq!(broken.current, 1, "серия начинается заново с 14 августа");
    assert_eq!(broken.freezes, 0, "запас сгорает вместе с серией");
    assert_eq!(mark(&broken, "2026-08-13"), Some(DayState::Missed));
}

#[test]
fn a_new_streak_earns_its_freezes_from_scratch() {
    let mut recorded = run_up_to("2026-06-30", 30);
    recorded.extend(run_up_to("2026-08-21", 5));

    let after = state(&recorded, "2026-08-21");

    assert_eq!(after.current, 5);
    assert_eq!(after.freezes, 0);
}

// --- рекорд ----------------------------------------------------------------

#[test]
fn the_longest_streak_is_remembered_with_the_days_it_ran() {
    let mut recorded = run_up_to("2026-04-10", 27);
    recorded.extend(run_up_to("2026-08-21", 5));

    let best = state(&recorded, "2026-08-21");

    assert_eq!(best.longest, 27);
    assert_eq!(best.longest_from.as_deref(), Some("2026-03-15"));
    assert_eq!(best.longest_to.as_deref(), Some("2026-04-10"));
}

#[test]
fn the_current_streak_can_be_the_record_itself() {
    let best = state(&run_up_to("2026-08-21", 12), "2026-08-21");

    assert_eq!(best.longest, 12);
    assert_eq!(best.current, 12);
    assert_eq!(best.longest_to.as_deref(), Some("2026-08-21"));
}

// --- разметка календаря ----------------------------------------------------

#[test]
fn every_day_of_the_run_is_marked() {
    let recorded = days(&[("2026-08-19", ENOUGH), ("2026-08-21", ENOUGH)]);

    let marked = state(&recorded, "2026-08-21");

    assert_eq!(mark(&marked, "2026-08-19"), Some(DayState::Counted));
    assert_eq!(mark(&marked, "2026-08-20"), Some(DayState::Missed));
    assert_eq!(mark(&marked, "2026-08-21"), Some(DayState::Counted));
    assert_eq!(mark(&marked, "2026-08-22"), None, "завтра ещё не размечено");
}

#[test]
fn the_marking_carries_the_seconds_of_each_day() {
    let recorded = days(&[("2026-08-21", 3 * 3600)]);

    let seconds = state(&recorded, "2026-08-21")
        .days
        .iter()
        .find(|day| day.day == "2026-08-21")
        .map(|day| day.seconds);

    assert_eq!(seconds, Some(3 * 3600));
}

// --- вехи ------------------------------------------------------------------

#[test]
fn the_milestones_are_seven_thirty_and_a_hundred() {
    let targets: Vec<u32> = milestones(&state(&days(&[]), "2026-08-21"))
        .iter()
        .map(|milestone| milestone.target)
        .collect();

    assert_eq!(targets, MILESTONES.to_vec());
}

#[test]
fn a_milestone_already_taken_remembers_the_day_it_was() {
    let taken = milestones(&state(&run_up_to("2026-08-21", 12), "2026-08-21"));

    assert!(taken[0].reached);
    assert_eq!(taken[0].reached_on.as_deref(), Some("2026-08-16"));
    assert_eq!(taken[0].remaining, 0);
}

#[test]
fn a_milestone_still_ahead_counts_the_days_left() {
    let ahead = milestones(&state(&run_up_to("2026-08-21", 12), "2026-08-21"));

    assert!(!ahead[1].reached);
    assert_eq!(ahead[1].remaining, 18);
    assert_eq!(ahead[1].reached_on, None);
    assert_eq!(ahead[2].remaining, 88);
}

#[test]
fn a_freeze_shifts_the_day_a_milestone_was_taken() {
    // Веха берётся на тридцатый занятый день, а не на тридцатый календарный:
    // замороженный день в счёт не идёт и сдвигает дату на сутки вперёд.
    let mut recorded = run_up_to("2026-08-10", 20);
    // 11 августа пропущено и закрыто заморозкой, дальше десять дней занятий.
    recorded.extend(run_up_to("2026-08-21", 10));

    let taken = milestones(&state(&recorded, "2026-08-21"));

    assert_eq!(state(&recorded, "2026-08-21").current, 30);
    assert!(taken[1].reached);
    assert_eq!(taken[1].reached_on.as_deref(), Some("2026-08-21"));
}

// --- календарь месяца ------------------------------------------------------

#[test]
fn the_calendar_covers_every_day_of_its_month() {
    let august = month_days(&state(&days(&[]), "2026-08-21"), "2026-08-21", 2026, 8);
    let february = month_days(&state(&days(&[]), "2026-08-21"), "2026-08-21", 2026, 2);

    assert_eq!(august.len(), 31);
    assert_eq!(august[0].day, "2026-08-01");
    assert_eq!(august[30].day, "2026-08-31");
    assert_eq!(february.len(), 28);
}

#[test]
fn the_calendar_tells_a_day_still_to_come_from_a_day_missed() {
    let recorded = days(&[("2026-08-20", ENOUGH)]);

    let august = month_days(&state(&recorded, "2026-08-21"), "2026-08-21", 2026, 8);
    let on = |day: u32| august[day as usize - 1].state;

    // До первого занятия — пусто, как и после сегодняшнего дня, но это
    // разные «пусто»: одно уже не наверстать, другое ещё впереди.
    assert_eq!(on(19), DayState::Missed);
    assert_eq!(on(20), DayState::Counted);
    assert_eq!(on(21), DayState::Pending);
    assert_eq!(on(22), DayState::Future);
    assert_eq!(on(31), DayState::Future);
}

#[test]
fn the_calendar_keeps_the_marks_the_walk_made() {
    let mut recorded = run_up_to("2026-08-10", 10);
    recorded.push(("2026-08-12".to_string(), ENOUGH));

    let august = month_days(&state(&recorded, "2026-08-12"), "2026-08-12", 2026, 8);

    assert_eq!(august[10].state, DayState::Frozen, "11 августа заморожено");
    assert_eq!(august[11].state, DayState::Counted);
}

#[test]
fn a_month_that_does_not_exist_has_no_days() {
    assert!(month_days(&state(&days(&[]), "2026-08-21"), "2026-08-21", 2026, 13).is_empty());
}
