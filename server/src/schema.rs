// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        machine_id -> Nullable<Text>,
    }
}

diesel::table! {
    environments (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
        project_id -> Integer,
    }
}

diesel::table! {
    environments_devices (id) {
        id -> Integer,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
        environment_id -> Integer,
        device_id -> Integer,
    }
}

diesel::table! {
    projects (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    states (id) {
        id -> Integer,
        json -> Text,
        created_at -> Timestamp,
        environment_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        access_token -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users_projects (id) {
        id -> Integer,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
        user_id -> Integer,
        project_id -> Integer,
    }
}

diesel::joinable!(environments -> projects (project_id));
diesel::joinable!(environments_devices -> devices (device_id));
diesel::joinable!(environments_devices -> environments (environment_id));
diesel::joinable!(states -> environments (environment_id));
diesel::joinable!(users_projects -> projects (project_id));
diesel::joinable!(users_projects -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    environments,
    environments_devices,
    projects,
    states,
    users,
    users_projects,
);
