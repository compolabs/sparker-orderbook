use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

mod limit_type;

pub use limit_type::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
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

#[cfg(feature = "with-sea")]
mod with_sea {
    use super::*;
    use sparker_entity::trade;
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
}

#[cfg(feature = "with-proto")]
mod with_proto {
    use super::*;
    use chrono::DateTime;
    use sparker_rpc::proto;

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
}
