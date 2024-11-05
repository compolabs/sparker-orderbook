use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sparker_entity::{sea_orm_active_enums, trade};
use sparker_rpc::proto;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum LimitType {
    GTC,
    IOC,
    FOK,
}

impl From<spark_market_sdk::LimitType> for LimitType {
    fn from(limit_type: spark_market_sdk::LimitType) -> Self {
        match limit_type {
            spark_market_sdk::LimitType::GTC => LimitType::GTC,
            spark_market_sdk::LimitType::IOC => LimitType::IOC,
            spark_market_sdk::LimitType::FOK => LimitType::FOK,
        }
    }
}

impl From<LimitType> for spark_market_sdk::LimitType {
    fn from(val: LimitType) -> Self {
        match val {
            LimitType::GTC => spark_market_sdk::LimitType::GTC,
            LimitType::IOC => spark_market_sdk::LimitType::IOC,
            LimitType::FOK => spark_market_sdk::LimitType::FOK,
        }
    }
}

impl From<sea_orm_active_enums::LimitType> for LimitType {
    fn from(limit_type: sea_orm_active_enums::LimitType) -> Self {
        match limit_type {
            sea_orm_active_enums::LimitType::Gtc => LimitType::GTC,
            sea_orm_active_enums::LimitType::Ioc => LimitType::IOC,
            sea_orm_active_enums::LimitType::Fok => LimitType::FOK,
        }
    }
}

impl From<LimitType> for sea_orm_active_enums::LimitType {
    fn from(limit_type: LimitType) -> Self {
        match limit_type {
            LimitType::GTC => sea_orm_active_enums::LimitType::Gtc,
            LimitType::IOC => sea_orm_active_enums::LimitType::Ioc,
            LimitType::FOK => sea_orm_active_enums::LimitType::Fok,
        }
    }
}

impl From<proto::LimitType> for LimitType {
    fn from(limit_type: proto::LimitType) -> Self {
        match limit_type {
            proto::LimitType::Gtc => LimitType::GTC,
            proto::LimitType::Ioc => LimitType::IOC,
            proto::LimitType::Fok => LimitType::FOK,
        }
    }
}

impl From<LimitType> for proto::LimitType {
    fn from(limit_type: LimitType) -> Self {
        match limit_type {
            LimitType::GTC => proto::LimitType::Gtc,
            LimitType::IOC => proto::LimitType::Ioc,
            LimitType::FOK => proto::LimitType::Fok,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Trade {
    pub tx_id: String,
    pub trade_id: String,
    pub order_id: String,
    pub limit_type: LimitType,
    pub size: u64,
    pub price: u64,
    pub timestamp: NaiveDateTime,
    pub market_id: String,
}

impl From<trade::Model> for Trade {
    fn from(trade: trade::Model) -> Self {
        Self {
            tx_id: trade.tx_id,
            trade_id: trade.trade_id,
            order_id: trade.order_id,
            limit_type: trade.limit_type.into(),
            size: trade.size as u64,
            price: trade.price as u64,
            timestamp: trade.timestamp,
            market_id: trade.market_id,
        }
    }
}

impl From<proto::Trade> for Trade {
    fn from(trade: proto::Trade) -> Self {
        let limit_type = proto::LimitType::from_repr(trade.limit_type).unwrap();

        Self {
            tx_id: trade.tx_id,
            trade_id: trade.trade_id,
            order_id: trade.order_id,
            limit_type: limit_type.into(),
            size: trade.size,
            price: trade.price,
            timestamp: DateTime::from_timestamp(trade.timestamp as i64, 0)
                .unwrap()
                .naive_utc(),
            market_id: trade.market_id,
        }
    }
}

impl From<Trade> for proto::Trade {
    fn from(trade: Trade) -> Self {
        Self {
            tx_id: trade.tx_id,
            trade_id: trade.trade_id,
            order_id: trade.order_id,
            limit_type: proto::LimitType::from(trade.limit_type) as i32,
            size: trade.size,
            price: trade.price,
            timestamp: trade.timestamp.and_utc().timestamp() as u64,
            market_id: trade.market_id,
        }
    }
}

pub type InsertTrade = Trade;
