use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::repo::{order, state, trade};

pub enum OperationMessage {
    Add(Operation),
    SetLatestProcessedBlock(i64),
    Dispatch,
}

pub enum Operation {
    InsertOrder(sparker_core::InsertOrder),
    UpdateOrder(sparker_core::UpdateOrder),
    InsertTrade(sparker_core::InsertTrade),
}

pub struct OperationDispatcher {
    db: Arc<DatabaseConnection>,
    operations: Vec<Operation>,
}

impl OperationDispatcher {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db,
            operations: vec![],
        }
    }

    pub async fn add(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub async fn set_latest_processed_block(&mut self, block: i64) {
        if let Err(e) = state::Mutation::upsert_latest_processed_block(&self.db, block).await {
            log::error!("SET_LATEST_PROCESSED_BLOCK_ERROR: {}", e);
        }
    }

    pub async fn dispatch(&mut self) {
        let create_orders = self.extract_operations(|operation| {
            if let Operation::InsertOrder(data) = operation {
                Some(data.clone())
            } else {
                None
            }
        });
        let update_orders = self.extract_operations(|operation| {
            if let Operation::UpdateOrder(data) = operation {
                Some(data.clone())
            } else {
                None
            }
        });
        let create_trades = self.extract_operations(|operation| {
            if let Operation::InsertTrade(data) = operation {
                Some(data.clone())
            } else {
                None
            }
        });

        // Insert many orders
        if let Err(e) = order::Mutation::insert_many(&self.db, create_orders).await {
            log::error!("CREATE_ORDERS_ERROR: {}", e);
        }

        // Update orders
        for order in update_orders {
            if let Err(e) = order::Mutation::update(&self.db, order).await {
                log::error!("UPDATE_ORDER_ERROR: {}", e);
            }
        }

        // Create trades
        if let Err(e) = trade::Mutation::insert_many(&self.db, create_trades).await {
            log::error!("CREATE_TRADES_ERROR: {}", e);
        }

        // Clear operations after dispatch
        self.operations.clear();
    }

    fn extract_operations<T, F>(&self, filter_fn: F) -> Vec<T>
    where
        F: Fn(&Operation) -> Option<T>,
    {
        self.operations.iter().filter_map(filter_fn).collect()
    }
}
