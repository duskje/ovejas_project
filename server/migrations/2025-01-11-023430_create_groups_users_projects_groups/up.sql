CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    name VARCHAR NOT NULL,
    access_token VARCHAR NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME
);

CREATE TABLE groups (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    name VARCHAR NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME
);

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

CREATE TABLE users_projects (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    deleted_at DATETIME,

    user_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,

    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(project_id) REFERENCES projects(id)
);
