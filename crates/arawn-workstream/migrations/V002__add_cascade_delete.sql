-- Add ON DELETE CASCADE to foreign keys
-- SQLite requires recreating tables to modify foreign key constraints

-- Step 1: Rename existing tables
ALTER TABLE sessions RENAME TO sessions_old;
ALTER TABLE workstream_tags RENAME TO workstream_tags_old;

-- Step 2: Create new tables with ON DELETE CASCADE
CREATE TABLE sessions (
    id             TEXT    PRIMARY KEY,
    workstream_id  TEXT    NOT NULL REFERENCES workstreams(id) ON DELETE CASCADE,
    started_at     TEXT    NOT NULL,
    ended_at       TEXT,
    turn_count     INTEGER,
    summary        TEXT,
    compressed     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE workstream_tags (
    workstream_id  TEXT NOT NULL REFERENCES workstreams(id) ON DELETE CASCADE,
    tag            TEXT NOT NULL,
    PRIMARY KEY (workstream_id, tag)
);

-- Step 3: Copy data from old tables
INSERT INTO sessions SELECT * FROM sessions_old;
INSERT INTO workstream_tags SELECT * FROM workstream_tags_old;

-- Step 4: Drop old tables
DROP TABLE sessions_old;
DROP TABLE workstream_tags_old;

-- Step 5: Recreate indexes
CREATE INDEX idx_sessions_workstream ON sessions(workstream_id);
