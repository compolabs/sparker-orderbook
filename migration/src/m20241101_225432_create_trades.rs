use sea_orm_migration::{prelude::*, schema::*, sea_orm::Iterable};

use crate::order::Order;
use crate::trade::{LimitTypeVariants, Trade};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Trade::Table)
                    .if_not_exists()
                    .col(pk_auto(Trade::Id))
                    .col(string(Trade::TxId))
                    .col(string_uniq(Trade::TradeId))
                    .col(string(Trade::OrderId))
                    .col(big_integer(Trade::Size))
                    .col(big_integer(Trade::Price))
                    .col(enumeration(
                        Trade::LimitType,
                        Alias::new("limit_type"),
                        LimitTypeVariants::iter(),
                    ))
                    .col(string(Trade::User))
                    .col(big_integer(Trade::BlockNumber))
                    .col(timestamp(Trade::Timestamp))
                    .col(string(Trade::MarketId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-trade-order_id")
                            .from(Trade::Table, Trade::OrderId)
                            .to(Order::Table, Order::OrderId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Trade::Table).to_owned())
            .await
    }
}
