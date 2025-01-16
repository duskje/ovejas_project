DROP TABLE states;

CREATE TABLE states (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    json VARCHAR NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    environment_id INTEGER NOT NULL,
    FOREIGN KEY(environment_id) REFERENCES environments(id)
);
