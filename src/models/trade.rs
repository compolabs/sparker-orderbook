use ::entity::trade::Model;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub type LimitType = ::entity::sea_orm_active_enums::LimitType;

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

impl From<Model> for Trade {
    fn from(trade: Model) -> Self {
        Self {
            tx_id: trade.tx_id,
            trade_id: trade.trade_id,
            order_id: trade.order_id,
            limit_type: trade.limit_type,
            size: trade.size as u64,
            price: trade.price as u64,
            timestamp: trade.timestamp,
            market_id: trade.market_id,
        }
    }
}

pub type CreateTrade = Trade;
