use chrono::Utc;
use sea_orm::{sea_query::OnConflict, DatabaseConnection, DbErr as Error, EntityTrait, Set};
use sparker_entity::state::{self, Entity as StateEntity};

pub struct Query;
impl Query {
    pub async fn find_latest_processed_block(
        db_conn: &DatabaseConnection,
    ) -> Result<Option<i64>, Error> {
        let state = StateEntity::find().one(db_conn).await?;

        Ok(state.map(|state| state.latest_processed_block))
    }
}

pub struct Mutation;
impl Mutation {
    pub async fn upsert_latest_processed_block(
        db_conn: &DatabaseConnection,
        block: i64,
    ) -> Result<(), Error> {
        let state = state::ActiveModel {
            id: Set(0),
            latest_processed_block: Set(block),
            timestamp: Set(Utc::now().naive_utc()),
        };

        let on_conflict = OnConflict::column(state::Column::Id)
            .update_column(state::Column::LatestProcessedBlock)
            .to_owned();
        StateEntity::insert(state)
            .on_conflict(on_conflict)
            .exec(db_conn)
            .await?;

        Ok(())
    }
}
