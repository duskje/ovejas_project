DROP TABLE group_devices;
DROP TABLE projects;

CREATE TABLE environments_devices (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME,

    environment_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,

    FOREIGN KEY(environment_id) REFERENCES environments(id),
    FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE projects (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    name VARCHAR NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME,

    environment_id INTEGER,
    FOREIGN KEY(environment_id) REFERENCES environments(id)
);
