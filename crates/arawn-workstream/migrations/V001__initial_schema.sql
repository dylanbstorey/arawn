-- Workstreams: persistent conversational contexts
CREATE TABLE workstreams (
    id            TEXT    PRIMARY KEY,
    title         TEXT    NOT NULL,
    summary       TEXT,
    is_scratch    INTEGER NOT NULL DEFAULT 0,
    state         TEXT    NOT NULL DEFAULT 'active',
    default_model TEXT,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL
);

-- Sessions: turn batches within a workstream
CREATE TABLE sessions (
    id             TEXT    PRIMARY KEY,
    workstream_id  TEXT    NOT NULL REFERENCES workstreams(id),
    started_at     TEXT    NOT NULL,
    ended_at       TEXT,
    turn_count     INTEGER,
    summary        TEXT,
    compressed     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_sessions_workstream ON sessions(workstream_id);

-- Tags: junction table for workstream labels
CREATE TABLE workstream_tags (
    workstream_id  TEXT NOT NULL REFERENCES workstreams(id),
    tag            TEXT NOT NULL,
    PRIMARY KEY (workstream_id, tag)
);
