use sea_orm::DatabaseConnection;
use sparker_core::{repo, LimitType, Order, OrderStatus, Trade, UpdateOrder};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::Receiver;

pub enum Operation {
    Update(Update),
    Dispatch(i64),
    Prune(i64),
}

pub enum Update {
    OpenOrder(sparker_core::Order),
    Trade(sparker_core::Trade),
    CancelOrder(String),
}

pub struct OperationDispatcher {
    market_id: String,
    db_conn: Arc<DatabaseConnection>,
    updates: Mutex<Vec<Update>>,
    operation_rx: Receiver<Operation>,
}

impl OperationDispatcher {
    pub fn new(
        market_id: String,
        db_conn: Arc<DatabaseConnection>,
        operation_rx: Receiver<Operation>,
    ) -> Self {
        Self {
            market_id,
            db_conn,
            updates: Mutex::new(Vec::new()),
            operation_rx,
        }
    }

    pub async fn start(&self) {
        while let Some(operation) = self.operation_rx.lock().await.recv().await {
            match operation {
                Operation::Update(update) => self.update(update).await,
                Operation::Dispatch(block) => self.dispatch(block).await,
                Operation::Prune(from_block) => self.prune(from_block).await,
            }
        }
    }

    /// Adds an update to the queue.
    ///
    /// # Arguments
    ///
    /// * `update` - The update to be added to the queue.
    pub async fn update(&self, update: Update) {
        let mut updates = self.updates.lock().await;
        updates.push(update);
    }

    /// Dispatches the updates for a given block.
    ///
    /// This method processes the updates by extracting open orders, cancel orders, and trades,
    /// and then processes them in the following order:
    /// 1. Open orders
    /// 2. Trade and update orders
    /// 3. Cancel orders
    ///
    /// After processing, it clears the updates and updates the latest processed block in the database.
    ///
    /// # Arguments
    ///
    /// * `block` - The block number to be processed.
    ///
    pub async fn dispatch(&self, block: i64) {
        let mut updates = self.updates.lock().await;
        let open_orders = extract_updates(&updates, |update| {
            if let Update::OpenOrder(data) = update {
                Some(data.clone())
            } else {
                None
            }
        });
        let cancel_order_ids = extract_updates(&updates, |update| {
            if let Update::CancelOrder(data) = update {
                Some(data.clone())
            } else {
                None
            }
        });
        let trades = extract_updates(&updates, |update| {
            if let Update::Trade(data) = update {
                Some(data.clone())
            } else {
                None
            }
        });

        self.process_open_orders(open_orders).await;
        self.process_trades(trades).await;
        self.process_cancel_orders(cancel_order_ids).await;

        // Clear operations after dispatch
        updates.clear();

        if let Err(e) = repo::state::Mutation::upsert_latest_processed_block(
            &self.db_conn,
            block,
            &self.market_id,
        )
        .await
        {
            log::error!("UPSERT_LATEST_PROCESSED_BLOCK_ERROR: {}", e);
        }
    }

    /// Prunes the orders and trades from the database.
    ///
    /// # Arguments
    ///
    /// * `from_block` - The block number to start pruning from.
    ///
    async fn prune(&self, from_block: i64) {
        if let Err(e) = repo::trade::Mutation::delete_many(&self.db_conn, self.market_id.clone(), from_block)
            .await
        {
            log::error!("PRUNE_TRADES_ERROR: {}", e);
        }

        if let Err(e) = repo::order::Mutation::delete_many(&self.db_conn, self.market_id.clone(), from_block)
            .await
        {
            log::error!("PRUNE_ORDERS_ERROR: {}", e);
        }
    }

    /// Processes the opening of orders by inserting them into the database.
    ///
    /// This method takes a vector of orders and attempts to insert them into the database.
    ///
    /// # Arguments
    ///
    /// * `orders` - A vector of orders to be inserted into the database.
    ///
    async fn process_open_orders(&self, orders: Vec<Order>) {
        if let Err(e) = repo::order::Mutation::insert_many(&self.db_conn, orders).await {
            log::error!("CREATE_ORDERS_ERROR: {}", e);
        }
    }

    /// Processes the cancellation of orders by updating their status to `Cancelled` in the database.
    ///
    /// For each order ID in the provided vector, it attempts to update the order's status to `Cancelled`.
    /// If an error occurs during the update, it logs the error.
    ///
    /// # Arguments
    ///
    /// * `order_ids` - A vector of order IDs to be cancelled.
    ///
    async fn process_cancel_orders(&self, order_ids: Vec<String>) {
        for order_id in order_ids {
            if let Err(e) = repo::order::Mutation::update(
                &self.db_conn,
                UpdateOrder {
                    order_id,
                    amount: None,
                    status: OrderStatus::Cancelled,
                },
            )
            .await
            {
                log::error!("CANCEL_ORDER_ERROR: {}", e);
            }
        }
    }

    /// Processes the given trades by updating the corresponding orders and inserting the trades into the database.
    ///
    /// For each trade, it finds the corresponding order by its ID. If the order is found, it updates the order's status
    /// and amount based on the trade's limit type. If the order is not found, it logs an error.
    ///
    /// After processing all trades, it inserts the trades into the database.
    ///
    /// # Arguments
    ///
    /// * `trades` - A vector of trades to be processed.
    ///
    async fn process_trades(&self, trades: Vec<Trade>) {
        for trade in trades.iter() {
            let order = match repo::order::Query::find_by_id(&self.db_conn, &trade.order_id).await {
                Ok(order) => order,
                Err(e) => {
                    log::error!("FIND_ORDER_BY_ID_ERROR: {}", e);
                    continue;
                }
            };

            match order {
                Some(order) => {
                    let (status, amount) = match trade.limit_type {
                        LimitType::GTC | LimitType::MKT => {
                            if order.amount > trade.size {
                                (OrderStatus::PartiallyMatched, Some(order.amount - trade.size))
                            } else {
                                (OrderStatus::Matched, None)
                            }
                        }
                        // FOK or IOC as fully matched
                        _ => (OrderStatus::Matched, None),
                    };

                    if let Err(e) = repo::order::Mutation::update(
                        &self.db_conn,
                        UpdateOrder {
                            order_id: trade.order_id.clone(),
                            amount,
                            status,
                        },
                    )
                    .await
                    {
                        log::error!("UPDATE_ORDER_ERROR: {}", e);
                    }
                }
                None => {
                    log::error!("ORDER_NOT_FOUND: {}", trade.order_id);
                }
            }
        }

        if let Err(e) = repo::trade::Mutation::insert_many(&self.db_conn, trades.clone()).await {
            log::error!("CREATE_TRADES_ERROR: {}", e);
        }
    }
}

fn extract_updates<T, F>(updates: &[Update], filter_fn: F) -> Vec<T>
where
    F: Fn(&Update) -> Option<T>,
{
    updates.iter().filter_map(filter_fn).collect()
}
