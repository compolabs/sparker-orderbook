use sea_orm_migration::{prelude::*, sea_orm::Iterable, sea_query::extension::postgres::Type};

use crate::{order::{OrderStatus, OrderStatusVariants, OrderType, OrderTypeVariants}, trade::{LimitType, LimitTypeVariants}};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(OrderType)
                    .values(OrderTypeVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(OrderStatus)
                    .values(OrderStatusVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(LimitType)
                    .values(LimitTypeVariants::iter())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_type(Type::drop().name(OrderType).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(OrderStatus).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(LimitType).to_owned())
            .await?;

        Ok(())
    }
}
