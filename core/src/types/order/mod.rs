use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

mod order_type;
mod status;

pub use order_type::*;
pub use status::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
pub struct Order {
    pub tx_id: String,
    pub order_id: String,
    pub order_type: OrderType,
    pub user: String,
    pub asset: String,
    pub amount: u64,
    pub price: u64,
    pub status: OrderStatus,
    pub timestamp: NaiveDateTime,
    pub market_id: String,
}

#[cfg(feature = "with-sea")]
mod with_sea {
    use super::*;
    use sparker_entity::order;
    impl From<order::Model> for Order {
        fn from(order: order::Model) -> Self {
            Self {
                tx_id: order.tx_id,
                order_id: order.order_id,
                order_type: order.order_type.into(),
                user: order.user,
                asset: order.asset,
                amount: order.amount as u64,
                price: order.price as u64,
                status: order.status.into(),
                timestamp: order.timestamp,
                market_id: order.market_id,
            }
        }
    }
}

#[cfg(feature = "with-proto")]
mod with_proto {
    use super::*;
    use chrono::DateTime;
    use sparker_proto::proto;

    impl From<proto::Order> for Order {
        fn from(order: proto::Order) -> Self {
            let order_type = proto::OrderType::from_repr(order.order_type).unwrap();
            let status = proto::OrderStatus::from_repr(order.status).unwrap();

            Self {
                tx_id: order.tx_id,
                order_id: order.order_id,
                order_type: order_type.into(),
                user: order.user,
                asset: order.asset,
                amount: order.amount,
                price: order.price,
                status: status.into(),
                timestamp: DateTime::from_timestamp(order.timestamp as i64, 0)
                    .unwrap()
                    .naive_utc(),
                market_id: order.market_id,
            }
        }
    }

    impl From<Order> for proto::Order {
        fn from(order: Order) -> Self {
            Self {
                tx_id: order.tx_id,
                order_id: order.order_id,
                order_type: proto::OrderType::from(order.order_type) as i32,
                user: order.user,
                asset: order.asset,
                amount: order.amount,
                price: order.price,
                status: proto::OrderStatus::from(order.status) as i32,
                timestamp: order.timestamp.and_utc().timestamp() as u64,
                market_id: order.market_id,
            }
        }
    }
}

impl Order {
    pub fn is_active(&self) -> bool {
        self.status == OrderStatus::New || self.status == OrderStatus::PartiallyMatched
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrder {
    pub order_id: String,
    pub amount: Option<u64>,
    pub status: OrderStatus,
}
