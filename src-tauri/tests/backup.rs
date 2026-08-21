//! Tests for the backup rotation rules: how a copy is named and which of
//! them are old enough to go.

use chrono::{TimeZone, Utc};
use lokked_lib::core::backup::{backup_name, stale, BACKUPS_KEPT};

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn a_copy_is_named_after_the_moment_it_was_taken() {
    let moment = Utc.with_ymd_and_hms(2026, 8, 21, 3, 45, 9).unwrap();

    assert_eq!(backup_name(moment), "lokked-20260821-034509.sqlite3");
}

#[test]
fn names_sort_chronologically_as_plain_text() {
    // На этом держится вся ротация: сортировка по имени — это сортировка по
    // времени, и никакого разбора даты не нужно.
    let earlier = backup_name(Utc.with_ymd_and_hms(2026, 8, 21, 3, 45, 9).unwrap());
    let later = backup_name(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());

    assert!(earlier < later);
}

#[test]
fn nothing_is_stale_while_there_is_room() {
    let existing = names(&["lokked-20260819-090000.sqlite3"]);

    assert_eq!(stale(&existing, 7), Vec::<String>::new());
}

#[test]
fn exactly_the_kept_number_is_still_not_stale() {
    let existing: Vec<String> = (1..=7)
        .map(|day| format!("lokked-202608{day:02}-090000.sqlite3"))
        .collect();

    assert_eq!(stale(&existing, 7), Vec::<String>::new());
}

#[test]
fn the_oldest_copies_go_first() {
    let existing: Vec<String> = (1..=9)
        .map(|day| format!("lokked-202608{day:02}-090000.sqlite3"))
        .collect();

    assert_eq!(
        stale(&existing, 7),
        names(&[
            "lokked-20260801-090000.sqlite3",
            "lokked-20260802-090000.sqlite3",
        ])
    );
}

#[test]
fn the_order_they_arrive_in_does_not_matter() {
    let existing = names(&[
        "lokked-20260803-090000.sqlite3",
        "lokked-20260801-090000.sqlite3",
        "lokked-20260802-090000.sqlite3",
    ]);

    assert_eq!(
        stale(&existing, 2),
        names(&["lokked-20260801-090000.sqlite3"])
    );
}

#[test]
fn files_that_are_not_ours_are_left_alone() {
    // В каталоге может лежать что угодно — заметка студента, копия, сделанная
    // руками. Чистка трогает только то, что создала сама.
    let existing = names(&[
        "заметка.txt",
        "lokked.sqlite3",
        "lokked-20260801-090000.sqlite3",
        "lokked-20260802-090000.sqlite3",
        "lokked-20260803-090000.sqlite3.bak",
    ]);

    // Наших копий две, и обе в пределах лимита; всё остальное в каталоге
    // чистка вообще не рассматривает.
    assert_eq!(stale(&existing, 2), Vec::<String>::new());
    assert_eq!(
        stale(&existing, 1),
        names(&["lokked-20260801-090000.sqlite3"])
    );
}

#[test]
fn keeping_none_means_every_copy_is_stale() {
    let existing = names(&["lokked-20260801-090000.sqlite3"]);

    assert_eq!(stale(&existing, 0), existing);
}

#[test]
fn seven_copies_are_kept_by_default() {
    assert_eq!(BACKUPS_KEPT, 7);
}
