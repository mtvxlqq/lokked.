//! Tests for schema migrations: applying from scratch and idempotency of a
//! repeat application. CRUD behaviour lives in `tests/db_repositories.rs`.
//!
//! `Database` deliberately exposes no raw `Connection` — callers work
//! through repositories, never rows (see `src-tauri/src/db/mod.rs`) — so
//! these tests exercise migrations through the public API rather than by
//! poking at `sqlite_master`/`PRAGMA user_version` directly.

use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;
use uuid::Uuid;

#[test]
fn opening_an_in_memory_database_applies_the_full_schema_from_scratch() {
    // `open_in_memory` runs every migration; if any statement in
    // `0001_init.sql` (subjects, timer_presets, sessions, decks, cards,
    // reviews, settings) were invalid, this would already fail here.
    let db = Database::open_in_memory().expect("open should apply the schema cleanly");

    // Exercise a table end to end as a sanity check beyond "it parsed".
    let subject = SubjectRepo::new(&db)
        .create("Math", None, None, 0)
        .expect("subjects table should be usable after migration");
    assert_eq!(subject.name, "Math");
}

#[test]
fn reopening_an_already_migrated_file_does_not_try_to_recreate_its_tables() {
    // The schema has no `IF NOT EXISTS`, so if migration bookkeeping ever
    // stopped gating on `PRAGMA user_version`, a second open would fail
    // with "table subjects already exists" instead of silently succeeding.
    let path = std::env::temp_dir().join(format!("lokked-test-{}.sqlite3", Uuid::now_v7()));

    let first_open = Database::open_at(&path).expect("first open should migrate from scratch");
    let subject = SubjectRepo::new(&first_open)
        .create("Physics", None, None, 0)
        .expect("insert into freshly migrated database");
    drop(first_open);

    let second_open =
        Database::open_at(&path).expect("reopening an up-to-date database must not fail");
    let found = SubjectRepo::new(&second_open)
        .get(&subject.id)
        .expect("query should succeed")
        .expect("data written before reopening should survive it");
    assert_eq!(found.name, "Physics");

    let _ = std::fs::remove_file(&path);
}
