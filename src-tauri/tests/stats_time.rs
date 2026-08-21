//! Tests for the time side of the statistics screen: the bars per subject,
//! the activity heatmap and the day range each of them covers.

use lokked_lib::core::stats::time::{
    heatmap, heatmap_start, range_start, subject_totals, StatsRange, HEAT_LEVELS,
};

/// `(subject_id, seconds)` pairs, as they come out of the database.
fn totals(pairs: &[(&str, i64)]) -> Vec<(String, i64)> {
    pairs
        .iter()
        .map(|(id, seconds)| ((*id).to_string(), *seconds))
        .collect()
}

#[test]
fn a_period_without_sessions_has_no_bars() {
    assert_eq!(subject_totals(&totals(&[])), Vec::new());
}

#[test]
fn the_longest_subject_comes_first_and_fills_the_bar() {
    let bars = subject_totals(&totals(&[("physics", 600), ("maths", 1800)]));

    assert_eq!(bars.len(), 2);
    assert_eq!(bars[0].subject_id, "maths");
    assert_eq!(bars[0].seconds, 1800);
    assert_eq!(bars[0].share_percent, 100);
    assert_eq!(bars[1].subject_id, "physics");
    assert_eq!(bars[1].share_percent, 33);
}

#[test]
fn repeated_rows_for_one_subject_are_summed() {
    let bars = subject_totals(&totals(&[("maths", 600), ("maths", 1200)]));

    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].seconds, 1800);
}

#[test]
fn a_subject_with_nothing_recorded_is_left_out() {
    // Ноль — это не столбик нулевой длины, а отсутствие строки: список
    // предметов у экрана и так есть, и рисовать пустые полоски незачем.
    let bars = subject_totals(&totals(&[("maths", 1800), ("physics", 0)]));

    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].subject_id, "maths");
}

#[test]
fn subjects_with_the_same_time_keep_a_stable_order() {
    let bars = subject_totals(&totals(&[("physics", 600), ("maths", 600)]));

    assert_eq!(bars[0].subject_id, "maths");
    assert_eq!(bars[1].subject_id, "physics");
    assert_eq!(bars[0].share_percent, 100);
    assert_eq!(bars[1].share_percent, 100);
}

#[test]
fn the_heatmap_covers_every_day_of_the_period() {
    let cells = heatmap(&totals(&[("2026-08-20", 3600)]), "2026-08-18", "2026-08-21");

    let days: Vec<&str> = cells.iter().map(|cell| cell.day_key.as_str()).collect();
    assert_eq!(
        days,
        ["2026-08-18", "2026-08-19", "2026-08-20", "2026-08-21"]
    );
}

#[test]
fn a_day_without_study_is_a_cell_at_level_zero() {
    let cells = heatmap(&totals(&[("2026-08-20", 3600)]), "2026-08-19", "2026-08-20");

    assert_eq!(cells[0].seconds, 0);
    assert_eq!(cells[0].level, 0);
    assert_eq!(cells[1].seconds, 3600);
    assert_eq!(cells[1].level, HEAT_LEVELS);
}

#[test]
fn levels_are_measured_against_the_best_day_of_the_period() {
    let recorded = totals(&[
        ("2026-08-18", 1000),
        ("2026-08-19", 2000),
        ("2026-08-20", 3000),
        ("2026-08-21", 4000),
    ]);

    let levels: Vec<u8> = heatmap(&recorded, "2026-08-18", "2026-08-21")
        .iter()
        .map(|cell| cell.level)
        .collect();

    assert_eq!(levels, [1, 2, 3, 4]);
}

#[test]
fn a_single_minute_still_shows_up() {
    // Иначе день с десятью минутами и день, когда учёбы не было, выглядят
    // одинаково — а разница между ними и есть весь смысл этой картинки.
    let recorded = totals(&[("2026-08-18", 60), ("2026-08-19", 36_000)]);

    let cells = heatmap(&recorded, "2026-08-18", "2026-08-19");

    assert_eq!(cells[0].level, 1);
}

#[test]
fn the_heatmap_knows_which_weekday_each_cell_is() {
    // 17 августа 2026 — понедельник.
    let cells = heatmap(&totals(&[]), "2026-08-17", "2026-08-23");

    let weekdays: Vec<u8> = cells.iter().map(|cell| cell.weekday).collect();
    assert_eq!(weekdays, [0, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn a_backwards_or_unparsable_period_gives_no_cells() {
    assert_eq!(heatmap(&totals(&[]), "2026-08-21", "2026-08-18").len(), 0);
    assert_eq!(heatmap(&totals(&[]), "вчера", "2026-08-18").len(), 0);
}

#[test]
fn rows_outside_the_period_are_ignored() {
    let recorded = totals(&[("2026-08-01", 36_000), ("2026-08-19", 600)]);

    let cells = heatmap(&recorded, "2026-08-18", "2026-08-19");

    assert_eq!(cells.len(), 2);
    // Уровень считается по дням периода, а не по выброшенному августу.
    assert_eq!(cells[1].level, HEAT_LEVELS);
}

#[test]
fn a_day_of_statistics_is_just_today() {
    assert_eq!(
        range_start(StatsRange::Day, "2026-08-21", None),
        "2026-08-21"
    );
}

#[test]
fn a_week_is_today_and_the_six_days_before_it() {
    assert_eq!(
        range_start(StatsRange::Week, "2026-08-21", None),
        "2026-08-15"
    );
}

#[test]
fn a_month_is_thirty_days_ending_today() {
    assert_eq!(
        range_start(StatsRange::Month, "2026-08-21", None),
        "2026-07-23"
    );
}

#[test]
fn all_time_starts_at_the_first_day_on_record() {
    assert_eq!(
        range_start(StatsRange::All, "2026-08-21", Some("2026-03-01")),
        "2026-03-01"
    );
}

#[test]
fn all_time_without_any_records_is_today() {
    assert_eq!(
        range_start(StatsRange::All, "2026-08-21", None),
        "2026-08-21"
    );
}

#[test]
fn a_first_record_in_the_future_does_not_invert_the_period() {
    // Часы на другом устройстве могли уйти вперёд; период всё равно должен
    // кончаться сегодняшним днём и начинаться не позже него.
    assert_eq!(
        range_start(StatsRange::All, "2026-08-21", Some("2027-01-01")),
        "2026-08-21"
    );
}

#[test]
fn ranges_survive_the_round_trip_through_their_slugs() {
    for range in [
        StatsRange::Day,
        StatsRange::Week,
        StatsRange::Month,
        StatsRange::All,
    ] {
        assert_eq!(StatsRange::parse(range.as_str()), Ok(range));
    }

    assert!(StatsRange::parse("десятилетие").is_err());
}

#[test]
fn the_heatmap_period_starts_on_a_monday() {
    // 21 августа 2026 — пятница; её неделя начинается 17-го.
    assert_eq!(heatmap_start("2026-08-21", 1), "2026-08-17");
}

#[test]
fn the_heatmap_counts_whole_weeks_back() {
    assert_eq!(heatmap_start("2026-08-21", 4), "2026-07-27");
}

#[test]
fn a_monday_is_the_start_of_its_own_week() {
    assert_eq!(heatmap_start("2026-08-17", 1), "2026-08-17");
}
