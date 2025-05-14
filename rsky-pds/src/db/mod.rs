use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use rocket_sync_db_pools::database;
use std::fmt::{Debug, Formatter};

#[database("pg_db")]
pub struct DbConn(PgConnection);

impl Debug for DbConn {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

#[tracing::instrument(skip_all)]
pub fn establish_connection_for_sequencer(db_url: &str) -> Result<PgConnection> {
    dotenv().ok();
    tracing::debug!("Establishing database connection for Sequencer");
    let database_url = db_url.to_string();
    let result = PgConnection::establish(&database_url).map_err(|error| {
        let context = format!("Error connecting to {database_url:?}");
        anyhow::Error::new(error).context(context)
    })?;

    Ok(result)
}
