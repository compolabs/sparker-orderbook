use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
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

#[cfg(feature = "with-sea")]
mod with_sea {
    use super::*;
    use sparker_entity::sea_orm_active_enums;
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
}

#[cfg(feature = "with-proto")]
mod with_proto {
    use super::*;
    use sparker_rpc::proto;

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
}
