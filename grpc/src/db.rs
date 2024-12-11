use sea_orm::{
    sqlx::{postgres::PgListener, Error as SqlxErr},
    Database, DatabaseConnection, DbErr,
};
use std::env;

pub async fn build_connection() -> Result<DatabaseConnection, DbErr> {
    let database_url = env::var("DATABASE_URL").unwrap();
    let db_conn: DatabaseConnection = Database::connect(database_url).await?;

    Ok(db_conn)
}

pub async fn build_listener(db_conn: &DatabaseConnection) -> Result<PgListener, SqlxErr> {
    let listener = PgListener::connect_with(db_conn.get_postgres_connection_pool()).await?;

    Ok(listener)
}
