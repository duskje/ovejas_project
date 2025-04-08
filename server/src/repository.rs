use std::convert::Infallible;

use diesel::prelude::*;
use deadpool_diesel::sqlite::Pool;
use diesel::result::Error::NotFound;

use crate::schema::{devices, environments, environments_devices, users, projects};
use crate::models::{Projects, Environments, Devices};



pub async fn device_create(device_name: String, machine_id: String, database_pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = database_pool.get().await.expect("Could not get database connection");

    conn.interact(move |conn| {
        diesel::insert_into(devices::table)
            .values((devices::name.eq(device_name), devices::machine_id.eq(machine_id)))
            .execute(conn)
    }).await??;

    Ok(())
}

pub async fn device_delete(machine_id: String, database_pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = database_pool.get().await.expect("Could not get database connection");

    conn.interact(move |conn| {
        let devices = devices::table;

        diesel::delete(devices.filter(devices::machine_id.like(machine_id)))
            .execute(conn)
    }).await??;

    Ok(())
}

pub async fn user_create(user_name: String, access_token: String, database_pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = database_pool.get().await.expect("Could not get database connection");

    conn.interact(move |conn| {
        diesel::insert_into(users::table)
            .values((users::name.eq(user_name), users::access_token.eq(access_token)))
            .execute(conn)
    }).await??;

    Ok(())
}

enum RepositoryError {
    ProjectNotFound,
    EnvironmentNotFound,
    DeviceNotFound,
}

pub async fn enroll_device_into_environment(
    machine_id: String,
    project_name: String,
    environment_name: String,
    database_pool: Pool
) -> Result<(), diesel::result::Error> {
    let conn = database_pool.get().await.expect("Could not get database connection");

    let result = conn.interact(|conn| -> Result<(), diesel::result::Error> {
        let project_result = projects::table
            .filter(projects::name.eq(project_name))
            .select(Projects::as_select())
            .get_result(conn);

        let project: Projects = match project_result {
            Ok(project) => project,
            Err(err) => return Err(err),
        };

        let environment_result = environments::table
            .filter(environments::name.eq(environment_name))
            .filter(environments::project_id.eq(project.id))
            .select(Environments::as_select())
            .get_result(conn);

        let environment: Environments = match environment_result {
            Ok(environment) => environment,
            Err(err) => return Err(err),
        };

        let device_result = devices::table
            .filter(devices::machine_id.eq(machine_id))
            .select(Devices::as_select())
            .get_result(conn);

        let device: Devices = match device_result {
            Ok(device) => device,
            Err(err) => return Err(err),
        };

        let insert_result = diesel::insert_into(environments_devices::table)
            .values((
                environments_devices::device_id.eq(device.id),
                environments_devices::environment_id.eq(environment.id),
            )).execute(conn);

        if let Err(err) = insert_result {
            Err(err)
        } else {
            Ok(())
        }
    }).await.unwrap();

    return result;
}
