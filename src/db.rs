use sea_orm::{Database, DatabaseConnection};
use std::env;

use crate::error::Error;

pub async fn build_connection() -> Result<DatabaseConnection, Error> {
    let database_url = env::var("DATABASE_URL").unwrap();
    let db_conn: DatabaseConnection = Database::connect(database_url).await?;

    Ok(db_conn)
}
