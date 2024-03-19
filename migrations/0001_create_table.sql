CREATE TABLE if NOT EXISTS sources (
    id INTEGER PRIMARY KEY,
    title TEXT,
    url TEXT,
    author TEXT,
    published_date DATE,
    viewed_date DATE,
    published_date_unknown BOOLEAN,
    comment TEXT
);