//! Handing the statistics over as a table.
//!
//! RFC 4180 with commas: a field is quoted only when it has to be, quotes
//! inside it are doubled, and rows end with a plain `\n` — the text goes to
//! the clipboard, and a spreadsheet reads either ending.

use super::percent;

/// One day of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyRow {
    pub day_key: String,
    pub seconds: i64,
    pub answered: u32,
    pub correct: u32,
}

/// The export the statistics screen offers: a line per day of the period.
///
/// Minutes are spelled out next to the seconds so the file is readable
/// without a formula, and accuracy is left empty on a day without cards —
/// «0%» there would claim everything was answered wrong.
pub fn daily_report(rows: &[DailyRow]) -> String {
    let body: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            vec![
                row.day_key.clone(),
                row.seconds.to_string(),
                (row.seconds / 60).to_string(),
                row.answered.to_string(),
                row.correct.to_string(),
                if row.answered == 0 {
                    String::new()
                } else {
                    percent(row.correct, row.answered).to_string()
                },
            ]
        })
        .collect();

    table(
        &[
            "день",
            "секунды",
            "минуты",
            "карточек",
            "верно",
            "точность %",
        ],
        &body,
    )
}

/// A header row and its body, as CSV.
pub fn table(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();

    out.push_str(&line(header.iter().map(|field| field.to_string())));
    for row in rows {
        out.push_str(&line(row.iter().cloned()));
    }

    out
}

fn line(fields: impl Iterator<Item = String>) -> String {
    let cells: Vec<String> = fields.map(|field| escape(&field)).collect();

    format!("{}\n", cells.join(","))
}

/// Quotes a field if the format requires it, doubling any quotes inside.
fn escape(field: &str) -> String {
    let needs_quotes =
        field.contains([',', '"', '\n', '\r']) || field.starts_with(' ') || field.ends_with(' ');

    if !needs_quotes {
        return field.to_string();
    }

    format!("\"{}\"", field.replace('"', "\"\""))
}
