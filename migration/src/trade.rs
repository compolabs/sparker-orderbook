use sea_orm_migration::{prelude::*, sea_orm::EnumIter};

#[derive(DeriveIden)]
pub struct LimitType;

#[derive(DeriveIden, EnumIter)]
pub enum LimitTypeVariants {
    GTC,
    IOC,
    FOK,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
pub enum Trade {
    Table,
    Id,
    TxId,
    TradeId,
    OrderId,
    LimitType,
    User,
    Size,
    Price,
    BlockNumber,
    Timestamp,
    MarketId,
}
