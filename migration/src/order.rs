use sea_orm_migration::{prelude::*, sea_orm::EnumIter};

#[derive(DeriveIden)]
pub struct OrderType;

#[derive(DeriveIden, EnumIter)]
pub enum OrderTypeVariants {
    Buy,
    Sell,
}

#[derive(DeriveIden)]
pub struct OrderStatus;

#[derive(DeriveIden, EnumIter)]
pub enum OrderStatusVariants {
    New,
    PartiallyMatched,
    Matched,
    Cancelled,
    Failed,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
pub enum Order {
    Table,
    Id,
    TxId,
    OrderId,
    OrderType,
    User,
    Asset,
    Amount,
    Price,
    Status,
    BlockNumber,
    Timestamp,
    MarketId,
}
