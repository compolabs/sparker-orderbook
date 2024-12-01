use chrono::Utc;
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DatabaseConnection, DbErr as Error, EntityTrait,
    QueryFilter, Set,
};
use sparker_entity::state::{self, Entity as StateEntity};

pub struct Query;
impl Query {
    pub async fn find_latest_processed_block(
        db_conn: &DatabaseConnection,
        market_id: &str,
    ) -> Result<Option<i64>, Error> {
        let state = StateEntity::find()
            .filter(state::Column::MarketId.eq(market_id))
            .one(db_conn)
            .await?;

        Ok(state.map(|state| state.latest_processed_block))
    }
}

pub struct Mutation;
impl Mutation {
    pub async fn upsert_latest_processed_block(
        db_conn: &DatabaseConnection,
        block: i64,
        market_id: &str,
    ) -> Result<(), Error> {
        let state = state::ActiveModel {
            market_id: Set(market_id.to_owned()),
            latest_processed_block: Set(block),
            timestamp: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let on_conflict = OnConflict::column(state::Column::MarketId)
            .update_column(state::Column::LatestProcessedBlock)
            .to_owned();
        StateEntity::insert(state)
            .on_conflict(on_conflict)
            .exec(db_conn)
            .await?;

        Ok(())
    }
}
