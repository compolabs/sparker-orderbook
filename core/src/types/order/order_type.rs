use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
pub enum OrderType {
    Buy,
    Sell,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<spark_market_sdk::OrderType> for OrderType {
    fn from(order_type: spark_market_sdk::OrderType) -> Self {
        match order_type {
            spark_market_sdk::OrderType::Buy => OrderType::Buy,
            spark_market_sdk::OrderType::Sell => OrderType::Sell,
        }
    }
}

impl From<OrderType> for spark_market_sdk::OrderType {
    fn from(val: OrderType) -> Self {
        match val {
            OrderType::Buy => spark_market_sdk::OrderType::Buy,
            OrderType::Sell => spark_market_sdk::OrderType::Sell,
        }
    }
}

#[cfg(feature = "with-sea")]
mod with_sea {
    use super::*;
    use sparker_entity::sea_orm_active_enums;

    impl From<sea_orm_active_enums::OrderType> for OrderType {
        fn from(order_type: sea_orm_active_enums::OrderType) -> Self {
            match order_type {
                sea_orm_active_enums::OrderType::Buy => OrderType::Buy,
                sea_orm_active_enums::OrderType::Sell => OrderType::Sell,
            }
        }
    }

    impl From<OrderType> for sea_orm_active_enums::OrderType {
        fn from(order_type: OrderType) -> Self {
            match order_type {
                OrderType::Buy => sea_orm_active_enums::OrderType::Buy,
                OrderType::Sell => sea_orm_active_enums::OrderType::Sell,
            }
        }
    }
}

#[cfg(feature = "with-proto")]
mod with_proto {
    use super::*;
    use sparker_proto::types as proto;

    impl From<proto::OrderType> for OrderType {
        fn from(order_type: proto::OrderType) -> Self {
            match order_type {
                proto::OrderType::Buy => OrderType::Buy,
                proto::OrderType::Sell => OrderType::Sell,
            }
        }
    }

    impl From<OrderType> for proto::OrderType {
        fn from(order_type: OrderType) -> Self {
            match order_type {
                OrderType::Buy => proto::OrderType::Buy,
                OrderType::Sell => proto::OrderType::Sell,
            }
        }
    }
}
