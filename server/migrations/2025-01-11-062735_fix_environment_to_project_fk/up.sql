DROP TABLE projects;

CREATE TABLE projects (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    name VARCHAR NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME
);

DROP TABLE environments;

CREATE TABLE environments (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    name VARCHAR NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME,

    project_id INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id)
);
