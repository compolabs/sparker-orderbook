use ::entity::order::Model;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub type OrderType = ::entity::sea_orm_active_enums::OrderType;
pub type OrderStatus = ::entity::sea_orm_active_enums::OrderStatus;

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

impl From<Model> for Order {
    fn from(order: Model) -> Self {
        Self {
            tx_id: order.tx_id,
            order_id: order.order_id,
            order_type: order.order_type,
            user: order.user,
            asset: order.asset,
            amount: order.amount as u64,
            price: order.price as u64,
            status: order.status,
            timestamp: order.timestamp,
            market_id: order.market_id,
        }
    }
}

pub type CreateOrder = Order;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrder {
    pub order_id: String,
    pub amount: Option<u64>,
    pub status: OrderStatus,
}
