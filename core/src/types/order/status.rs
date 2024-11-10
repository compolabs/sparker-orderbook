use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
pub enum OrderStatus {
    Cancelled,
    Failed,
    Matched,
    New,
    PartiallyMatched,
}

#[cfg(feature = "with-sea")]
mod with_sea {
    use super::*;
    use sparker_entity::sea_orm_active_enums;

    impl From<sea_orm_active_enums::OrderStatus> for OrderStatus {
        fn from(order_status: sea_orm_active_enums::OrderStatus) -> Self {
            match order_status {
                sea_orm_active_enums::OrderStatus::Cancelled => OrderStatus::Cancelled,
                sea_orm_active_enums::OrderStatus::Failed => OrderStatus::Failed,
                sea_orm_active_enums::OrderStatus::Matched => OrderStatus::Matched,
                sea_orm_active_enums::OrderStatus::New => OrderStatus::New,
                sea_orm_active_enums::OrderStatus::PartiallyMatched => {
                    OrderStatus::PartiallyMatched
                }
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
                OrderStatus::PartiallyMatched => {
                    sea_orm_active_enums::OrderStatus::PartiallyMatched
                }
            }
        }
    }
}

#[cfg(feature = "with-proto")]
mod with_proto {
    use super::*;
    use sparker_proto::proto;

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
}
