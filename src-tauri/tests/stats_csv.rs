//! Tests for the CSV export: escaping, and the daily report it builds.

use lokked_lib::core::stats::csv::{daily_report, table, DailyRow};

fn row(day_key: &str, seconds: i64, answered: u32, correct: u32) -> DailyRow {
    DailyRow {
        day_key: day_key.to_string(),
        seconds,
        answered,
        correct,
    }
}

#[test]
fn plain_fields_are_written_as_they_are() {
    let csv = table(
        &["день", "минут"],
        &[vec!["2026-08-21".into(), "42".into()]],
    );

    assert_eq!(csv, "день,минут\n2026-08-21,42\n");
}

#[test]
fn a_field_with_a_comma_is_quoted() {
    let csv = table(&["предмет"], &[vec!["Алгебра, часть 2".into()]]);

    assert_eq!(csv, "предмет\n\"Алгебра, часть 2\"\n");
}

#[test]
fn a_quote_inside_a_field_is_doubled() {
    let csv = table(
        &["карточка"],
        &[vec![r#"Теорема "о двух милиционерах""#.into()]],
    );

    assert_eq!(csv, "карточка\n\"Теорема \"\"о двух милиционерах\"\"\"\n");
}

#[test]
fn a_line_break_inside_a_field_stays_inside_the_quotes() {
    let csv = table(&["ответ"], &[vec!["первая\nвторая".into()]]);

    assert_eq!(csv, "ответ\n\"первая\nвторая\"\n");
}

#[test]
fn a_table_without_rows_is_still_a_header() {
    assert_eq!(table(&["день"], &[]), "день\n");
}

#[test]
fn the_daily_report_counts_minutes_and_accuracy() {
    let csv = daily_report(&[row("2026-08-21", 5400, 4, 3)]);
    let lines: Vec<&str> = csv.lines().collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1], "2026-08-21,5400,90,4,3,75");
}

#[test]
fn a_day_without_cards_reports_no_accuracy_rather_than_zero_per_cent() {
    let csv = daily_report(&[row("2026-08-21", 600, 0, 0)]);
    let lines: Vec<&str> = csv.lines().collect();

    // Пустое поле, а не «0»: ноль процентов означал бы, что всё отвечено
    // неверно, а карточек в этот день просто не было.
    assert_eq!(lines[1], "2026-08-21,600,10,0,0,");
}
