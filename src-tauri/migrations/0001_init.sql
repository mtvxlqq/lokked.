-- Initial schema. See docs/PLAN.md section 1.
--
-- Primary keys are UUIDv7 (TEXT), generated application-side — no AUTOINCREMENT.
-- Every table (except the append-only ones) carries created_at/updated_at/
-- deleted_at; deletion is always soft (deleted_at = now), never a real DELETE.
-- `sessions` and `reviews` are append-only: no deleted_at, rows are never
-- edited once written.
--
-- This file is never edited once shipped. A schema change is a new migration.

CREATE TABLE subjects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    icon TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE timer_presets (
    id TEXT PRIMARY KEY,
    subject_id TEXT REFERENCES subjects(id), -- NULL = global preset, applies to every subject
    name TEXT NOT NULL,
    mode TEXT NOT NULL,                      -- 'countup' | 'countdown' | 'pomodoro'
    work_seconds INTEGER NOT NULL,
    break_seconds INTEGER,
    long_break_seconds INTEGER,
    cycles_before_long INTEGER,
    auto_start_next INTEGER NOT NULL DEFAULT 0,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE sessions ( -- append-only: no deleted_at, rows are never edited
    id TEXT PRIMARY KEY,
    subject_id TEXT NOT NULL REFERENCES subjects(id),
    preset_id TEXT REFERENCES timer_presets(id),
    mode TEXT NOT NULL,
    phase TEXT NOT NULL,                     -- 'work' | 'break' | 'long_break'
    started_at TEXT NOT NULL,                -- UTC
    ended_at TEXT NOT NULL,                  -- UTC
    day_key TEXT NOT NULL,                   -- 'YYYY-MM-DD', from core::dayline
    active_seconds INTEGER NOT NULL,
    paused_seconds INTEGER NOT NULL DEFAULT 0,
    planned_seconds INTEGER,
    completed INTEGER NOT NULL DEFAULT 0,
    interruptions INTEGER NOT NULL DEFAULT 0,
    device_id TEXT,                          -- for future sync (M16)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_sessions_day ON sessions(day_key);
CREATE INDEX idx_sessions_subject ON sessions(subject_id);

CREATE TABLE decks (
    id TEXT PRIMARY KEY,
    subject_id TEXT REFERENCES subjects(id),
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    deck_id TEXT NOT NULL REFERENCES decks(id),
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    hint TEXT,
    tags TEXT,
    -- Adaptive scheduling fields (M17): a cache of weights computed from
    -- `reviews`, not a state machine of their own. NULL until first reviewed.
    ease REAL,
    interval_days REAL,
    due_at TEXT,
    reps INTEGER,
    lapses INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE reviews ( -- append-only: no deleted_at, rows are never edited
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id),
    reviewed_at TEXT NOT NULL,
    day_key TEXT NOT NULL,
    result TEXT NOT NULL,                    -- 'again' | 'hard' | 'good' | 'easy'
    correct INTEGER NOT NULL,                -- 0/1, derived, kept for cheap stats
    mode TEXT NOT NULL,                      -- 'classic' | 'blitz' | 'marathon' | 'weak'
    think_ms INTEGER,
    total_ms INTEGER,
    device_id TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_reviews_card ON reviews(card_id);
CREATE INDEX idx_reviews_day ON reviews(day_key);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TEXT NOT NULL
);
