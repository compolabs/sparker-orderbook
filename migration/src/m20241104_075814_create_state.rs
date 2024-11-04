use sea_orm_migration::{prelude::*, schema::*};

use crate::state::State;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(State::Table)
                    .if_not_exists()
                    .col(pk_auto(State::Id))
                    .col(big_integer(State::LatestProcessedBlock))
                    .col(timestamp(State::Timestamp))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(State::Table).to_owned())
            .await
    }
}
