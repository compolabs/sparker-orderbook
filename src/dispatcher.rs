use std::sync::Arc;

use crate::{
    models::{CreateOrder, CreateTrade, UpdateOrder},
    repos::{OrderRepository, StateRepository, TradeRepository},
};

pub enum OperationMessage {
    Add(Operation),
    SetLatestProcessedBlock(i64),
    Dispatch,
}

pub enum Operation {
    CreateOrder { data: CreateOrder },
    UpdateOrder { data: UpdateOrder },
    CreateTrade { data: CreateTrade },
}

pub struct OperationDispatcher {
    order_repository: Arc<OrderRepository>,
    trade_repository: Arc<TradeRepository>,
    state_repository: Arc<StateRepository>,
    operations: Vec<Operation>,
}

impl OperationDispatcher {
    pub fn new(
        order_repository: Arc<OrderRepository>,
        trade_repository: Arc<TradeRepository>,
        state_repository: Arc<StateRepository>,
    ) -> Self {
        Self {
            order_repository,
            trade_repository,
            state_repository,
            operations: vec![],
        }
    }

    pub async fn add(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub async fn set_latest_processed_block(&mut self, block: i64) {
        if let Err(e) = self
            .state_repository
            .upsert_latest_processed_block(block)
            .await
        {
            log::error!("SET_LATEST_PROCESSED_BLOCK_ERROR: {}", e);
        }
    }

    pub async fn dispatch(&mut self) {
        let create_orders = self.extract_operations(|operation| {
            if let Operation::CreateOrder { data } = operation {
                Some(data.clone())
            } else {
                None
            }
        });
        let update_orders = self.extract_operations(|operation| {
            if let Operation::UpdateOrder { data } = operation {
                Some(data.clone())
            } else {
                None
            }
        });
        let create_trades = self.extract_operations(|operation| {
            if let Operation::CreateTrade { data } = operation {
                Some(data.clone())
            } else {
                None
            }
        });

        // Insert many orders
        if let Err(e) = self.order_repository.create_orders(create_orders).await {
            log::error!("CREATE_ORDERS_ERROR: {}", e);
        }

        // Update orders
        for order in update_orders {
            if let Err(e) = self
                .order_repository
                .update_order(&order.order_id.clone(), order)
                .await
            {
                log::error!("UPDATE_ORDER_ERROR: {}", e);
            }
        }

        // Create trades
        if let Err(e) = self.trade_repository.create_trades(create_trades).await {
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
