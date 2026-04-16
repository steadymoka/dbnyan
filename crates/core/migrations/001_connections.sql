CREATE TABLE connections (
    id             TEXT PRIMARY KEY NOT NULL,
    name           TEXT NOT NULL,
    driver         TEXT NOT NULL,
    host           TEXT NOT NULL,
    port           INTEGER NOT NULL,
    username       TEXT NOT NULL,
    password       TEXT,
    database_name  TEXT,
    folder         TEXT,
    ssh_json       TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE INDEX idx_connections_folder ON connections(folder);
