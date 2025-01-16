-- This file should undo anything in `up.sql`
DROP TABLE environments_devices;
DROP TABLE projects;

CREATE TABLE group_devices (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME,

    group_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,

    FOREIGN KEY(group_id) REFERENCES groups(id),
    FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE projects (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    name VARCHAR NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME,

    group_id INTEGER,
    FOREIGN KEY(group_id) REFERENCES groups(id)
);
