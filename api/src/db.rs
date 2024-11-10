use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;

pub async fn build_connection() -> Result<DatabaseConnection, DbErr> {
    let database_url = env::var("DATABASE_URL").unwrap();
    let db: DatabaseConnection = Database::connect(database_url).await?;

    Ok(db)
}
