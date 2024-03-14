CREATE TABLE if NOT EXISTS sources (
    id INTEGER PRIMARY KEY,
    url TEXT NOT NULL,
    author TEXT,
    date DATE
);