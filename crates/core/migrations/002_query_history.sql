CREATE TABLE query_history (
    id              TEXT PRIMARY KEY NOT NULL,
    connection_id   TEXT NOT NULL REFERENCES connections(id) ON DELETE CASCADE,
    database_name   TEXT,
    sql             TEXT NOT NULL,
    success         INTEGER NOT NULL,
    error           TEXT,
    rows_affected   INTEGER,
    rows_returned   INTEGER,
    duration_ms     INTEGER NOT NULL,
    executed_at     TEXT NOT NULL
);

CREATE INDEX idx_history_conn_executed
    ON query_history(connection_id, executed_at DESC);
