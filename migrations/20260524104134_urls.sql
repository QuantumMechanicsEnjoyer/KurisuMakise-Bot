-- Add migration script here

CREATE TABLE urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    discord_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    description TEXT,
    created_at INTEGER NOT NULL
);