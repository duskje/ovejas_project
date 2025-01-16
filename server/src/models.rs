use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug)]
#[diesel(table_name = crate::schema::states)]
#[diesel(belongs_to(Environments, foreign_key = environment_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct States {
    pub id: i32,
    pub json: String,
    pub created_at: NaiveDateTime,
    pub environment_id: i32,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Devices {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Projects {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Users {
    pub id: i32,
    pub name: String,
    pub access_token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(Projects, foreign_key = project_id))]
#[diesel(table_name = crate::schema::environments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Environments {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
    pub project_id: i32,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Debug)]
#[diesel(belongs_to(Devices, foreign_key = device_id))]
#[diesel(belongs_to(Environments, foreign_key = environment_id))]
#[diesel(table_name = crate::schema::environments_devices)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DevicesEnvironments {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
    pub device_id: i32,
    pub environment_id: i32,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Debug)]
#[diesel(belongs_to(Users, foreign_key = user_id))]
#[diesel(belongs_to(Projects, foreign_key = project_id))]
#[diesel(table_name = crate::schema::users_projects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UsersProjects {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
    pub user_id: i32,
    pub project_id: i32,
}
