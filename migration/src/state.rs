use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum State {
    Table,
    Id,
    MarketId,
    LatestProcessedBlock,
    Timestamp,
}
