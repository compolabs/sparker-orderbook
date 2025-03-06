use chrono::DateTime;
use ethers_core::k256::sha2::{Digest, Sha256};
use rustc_hex::ToHex;
use serde::{Deserialize, Serialize};
use sparker_core::{LimitType, Order, OrderStatus, OrderType, Trade};

#[derive(Debug, Deserialize, Serialize)]
pub struct PangeaEvent {
    pub chain: u64,
    pub block_number: i64,
    pub block_hash: String,
    pub block_timestamp: i64,
    pub transaction_hash: String,
    pub transaction_index: u64,
    pub log_index: u64,
    pub market_id: String,
    pub order_id: String,
    pub event_type: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<u128>,
    pub asset_type: Option<String>,
    pub order_type: Option<String>,
    pub price: Option<u128>,
    pub user: Option<String>,
    pub order_matcher: Option<String>,
    pub owner: Option<String>,
    pub limit_type: Option<String>,
}

impl PangeaEvent {
    pub fn order_type(&self) -> Option<OrderType> {
        self.order_type
            .as_deref()
            .and_then(|order_type| match order_type {
                "Buy" => Some(OrderType::Buy),
                "Sell" => Some(OrderType::Sell),
                _ => None,
            })
    }

    pub fn limit_type(&self) -> LimitType {
        match self.limit_type.as_deref() {
            Some("FOK") => LimitType::FOK,
            Some("IOC") => LimitType::IOC,
            Some("MKT") => LimitType::MKT,
            _ => LimitType::GTC,
        }
    }

    pub fn trade_id(&self) -> String {
        let hex: String = Sha256::digest(
            format!(
                "{}{}{}{}{}",
                self.transaction_hash,
                self.order_id,
                self.block_timestamp,
                self.amount.unwrap(),
                self.log_index,
            )
            .as_bytes(),
        )
        .to_hex();

        format!("0x{}", hex)
    }

    pub fn build_order(&self) -> Option<Order> {
        if let (Some(price), Some(amount), Some(user)) = (self.price, self.amount, &self.user) {
            let order_type = self.order_type().unwrap();

            Some(Order {
                tx_id: self.transaction_hash.clone(),
                order_id: self.order_id.clone(),
                order_type,
                user: user.to_owned(),
                asset: self.asset.as_deref().unwrap().to_owned(),
                amount: amount as u64,
                price: price as u64,
                status: OrderStatus::New,
                block_number: self.block_number as u64,
                timestamp: DateTime::from_timestamp(self.block_timestamp, 0)
                    .unwrap()
                    .naive_utc(),
                market_id: self.market_id.clone(),
            })
        } else {
            None
        }
    }

    pub fn build_trade(&self) -> Option<Trade> {
        if let (Some(price), Some(amount)) = (self.price, self.amount) {
            Some(Trade {
                tx_id: self.transaction_hash.clone(),
                trade_id: self.trade_id(),
                order_id: self.order_id.clone(),
                limit_type: self.limit_type(),
                user: self.user.as_deref().unwrap().to_owned(),
                size: amount as u64,
                price: price as u64,
                block_number: self.block_number as u64,
                timestamp: DateTime::from_timestamp(self.block_timestamp, 0)
                    .unwrap()
                    .naive_utc(),
                market_id: self.market_id.clone(),
            })
        } else {
            None
        }
    }
}
