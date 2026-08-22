-- Duels: a blitz two to four people take turns at, on one device.
--
-- Same conventions as 0001: UUIDv7 text keys, created_at/updated_at/
-- deleted_at, soft deletion, and one append-only table for the answers.
--
-- The point of storing them apart from `reviews` is whose history they are.
-- A guest's answers are not the owner's study record and must never reach the
-- statistics screen or the card picker, so they live here and nowhere else.
-- The owner's own answers are written to `reviews` as well — they studied.

CREATE TABLE duels (
    id TEXT PRIMARY KEY,
    deck_id TEXT NOT NULL REFERENCES decks(id),
    day_key TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,                        -- NULL while the duel is still going
    cards INTEGER NOT NULL,                  -- how many cards each player answers
    seconds_per_card INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE duel_players (
    id TEXT PRIMARY KEY,
    duel_id TEXT NOT NULL REFERENCES duels(id),
    name TEXT NOT NULL,
    position INTEGER NOT NULL,               -- turn order, 0-based
    is_owner INTEGER NOT NULL DEFAULT 0,     -- 1 for the student whose device it is
    points INTEGER NOT NULL DEFAULT 0,
    correct INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE duel_answers ( -- append-only: no deleted_at, rows are never edited
    id TEXT PRIMARY KEY,
    duel_id TEXT NOT NULL REFERENCES duels(id),
    player_id TEXT NOT NULL REFERENCES duel_players(id),
    card_id TEXT NOT NULL REFERENCES cards(id),
    position INTEGER NOT NULL,               -- which card of the shared sequence
    result TEXT NOT NULL,                    -- 'again' | 'hard' | 'good' | 'easy'
    correct INTEGER NOT NULL,                -- 0/1, derived, kept for cheap stats
    total_ms INTEGER,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_duels_day ON duels(day_key);
CREATE INDEX idx_duel_players_duel ON duel_players(duel_id);
CREATE INDEX idx_duel_answers_duel ON duel_answers(duel_id);
