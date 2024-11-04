use ::entity::state::{self, Entity as StateEntity};
use chrono::Utc;
use sea_orm::{sea_query::OnConflict, DatabaseConnection, EntityTrait, Set};

use crate::error::Error;

pub struct StateRepository {
    conn: DatabaseConnection,
}

impl StateRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn latest_processed_block(&self) -> Result<Option<i64>, Error> {
        let state = StateEntity::find().one(&self.conn).await?;

        Ok(state.map(|state| state.latest_processed_block))
    }

    pub async fn upsert_latest_processed_block(
        &self,
        latest_processed_block: i64,
    ) -> Result<(), Error> {
        let state = state::ActiveModel {
            id: Set(0),
            latest_processed_block: Set(latest_processed_block),
            timestamp: Set(Utc::now().naive_utc()),
        };
        let on_conflict = OnConflict::column(state::Column::Id)
            .update_column(state::Column::LatestProcessedBlock)
            .to_owned();
        StateEntity::insert(state)
            .on_conflict(on_conflict)
            .exec(&self.conn)
            .await?;

        Ok(())
    }
}
