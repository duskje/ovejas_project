use diesel::prelude::*;
use deadpool_diesel::sqlite::Pool;

use crate::schema::devices;

pub async fn device_create(name: String, machine_id: String, database_pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = database_pool.get().await.expect("Could not get database connection");

    conn.interact(move |conn| {
        diesel::insert_into(devices::table)
            .values((devices::name.eq(name), devices::machine_id.eq(machine_id)))
            .execute(conn)
    }).await??;

    Ok(())
}

pub async fn user_create(name: String, machine_id: String, database_pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = database_pool.get().await.expect("Could not get database connection");

    conn.interact(move |conn| {
        diesel::insert_into(devices::table)
            .values((devices::name.eq(name), devices::machine_id.eq(machine_id)))
            .execute(conn)
    }).await??;

    Ok(())
}
