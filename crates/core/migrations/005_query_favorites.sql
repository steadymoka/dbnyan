CREATE TABLE query_favorites (
    id              TEXT PRIMARY KEY NOT NULL,
    connection_id   TEXT NOT NULL REFERENCES connections(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    sql             TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX idx_favorites_conn_name ON query_favorites(connection_id, name);
