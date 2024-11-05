use chrono::DateTime;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sparker_entity::order;
use sparker_entity::sea_orm_active_enums;
use sparker_rpc::proto;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum OrderType {
    Buy,
    Sell,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum OrderStatus {
    Cancelled,
    Failed,
    Matched,
    New,
    PartiallyMatched,
}

impl From<sea_orm_active_enums::OrderStatus> for OrderStatus {
    fn from(order_status: sea_orm_active_enums::OrderStatus) -> Self {
        match order_status {
            sea_orm_active_enums::OrderStatus::Cancelled => OrderStatus::Cancelled,
            sea_orm_active_enums::OrderStatus::Failed => OrderStatus::Failed,
            sea_orm_active_enums::OrderStatus::Matched => OrderStatus::Matched,
            sea_orm_active_enums::OrderStatus::New => OrderStatus::New,
            sea_orm_active_enums::OrderStatus::PartiallyMatched => OrderStatus::PartiallyMatched,
        }
    }
}

impl From<OrderStatus> for sea_orm_active_enums::OrderStatus {
    fn from(order_status: OrderStatus) -> Self {
        match order_status {
            OrderStatus::Cancelled => sea_orm_active_enums::OrderStatus::Cancelled,
            OrderStatus::Failed => sea_orm_active_enums::OrderStatus::Failed,
            OrderStatus::Matched => sea_orm_active_enums::OrderStatus::Matched,
            OrderStatus::New => sea_orm_active_enums::OrderStatus::New,
            OrderStatus::PartiallyMatched => sea_orm_active_enums::OrderStatus::PartiallyMatched,
        }
    }
}

impl From<proto::OrderStatus> for OrderStatus {
    fn from(order_status: proto::OrderStatus) -> Self {
        match order_status {
            proto::OrderStatus::Cancelled => OrderStatus::Cancelled,
            proto::OrderStatus::Failed => OrderStatus::Failed,
            proto::OrderStatus::Matched => OrderStatus::Matched,
            proto::OrderStatus::New => OrderStatus::New,
            proto::OrderStatus::PartiallyMatched => OrderStatus::PartiallyMatched,
        }
    }
}

impl From<OrderStatus> for proto::OrderStatus {
    fn from(order_status: OrderStatus) -> Self {
        match order_status {
            OrderStatus::Cancelled => proto::OrderStatus::Cancelled,
            OrderStatus::Failed => proto::OrderStatus::Failed,
            OrderStatus::Matched => proto::OrderStatus::Matched,
            OrderStatus::New => proto::OrderStatus::New,
            OrderStatus::PartiallyMatched => proto::OrderStatus::PartiallyMatched,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

pub type InsertOrder = Order;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrder {
    pub order_id: String,
    pub amount: Option<u64>,
    pub status: OrderStatus,
}
