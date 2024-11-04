use sea_orm_migration::{prelude::*, schema::*, sea_orm::Iterable};

use crate::order::{Order, OrderStatusVariants, OrderTypeVariants};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .if_not_exists()
                    .col(pk_auto(Order::Id))
                    .col(string(Order::TxId))
                    .col(string_uniq(Order::OrderId))
                    .col(enumeration(
                        Order::OrderType,
                        Alias::new("order_type"),
                        OrderTypeVariants::iter(),
                    ))
                    .col(string(Order::User))
                    .col(string(Order::Asset))
                    .col(big_integer(Order::Amount))
                    .col(big_integer(Order::Price))
                    .col(enumeration(
                        Order::Status,
                        Alias::new("order_status"),
                        OrderStatusVariants::iter(),
                    ))
                    .col(timestamp(Order::Timestamp))
                    .col(string(Order::MarketId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Order::Table).to_owned())
            .await
    }
}
