use std::sync::Arc;

use dotenv::dotenv;
use error::Error;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{mpsc::unbounded_channel, Mutex},
};

use crate::{
    api::AppState,
    config::Config,
    dispatcher::{OperationDispatcher, OperationMessage},
    pangea::PangeaIndexer,
    repos::{OrderRepository, StateRepository, TradeRepository},
    store::Store,
};

mod api;
mod config;
mod db;
mod dispatcher;
mod error;
mod models;
mod pangea;
mod repos;
mod store;
mod types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();

    let config = Config::load("config.testnet.json")?;

    let db_conn = db::build_connection().await?;
    let order_repository = Arc::new(OrderRepository::new(db_conn.clone()));
    let trade_repository = Arc::new(TradeRepository::new(db_conn.clone()));
    let state_repository = Arc::new(StateRepository::new(db_conn.clone()));

    let latest_processed_block = state_repository
        .latest_processed_block()
        .await?
        .unwrap_or(config.pangea_start_block);
    let orders = order_repository.orders(10_000, 0).await?;

    log::debug!("Orders: {:?}", orders.len());

    // Local store for order and trades
    let store = Arc::new(Mutex::new(Store::new(orders)));

    // ------------------ Start dispatcher ------------------
    let mut operation_dispatcher = OperationDispatcher::new(
        Arc::clone(&order_repository),
        Arc::clone(&trade_repository),
        Arc::clone(&state_repository),
    );
    let (operation_tx, operation_rx) = unbounded_channel::<OperationMessage>();
    let operation_tx = Arc::new(operation_tx);
    let operation_rx = Arc::new(Mutex::new(operation_rx));

    tokio::spawn(async move {
        while let Some(message) = operation_rx.lock().await.recv().await {
            match message {
                OperationMessage::Add(operation) => operation_dispatcher.add(operation).await,
                OperationMessage::SetLatestProcessedBlock(block) => {
                    operation_dispatcher.set_latest_processed_block(block).await
                }
                OperationMessage::Dispatch => operation_dispatcher.dispatch().await,
            }
        }
    });

    // ------------------ Start indexer ------------------
    log::info!("Starting indexer...");
    let indexer =
        PangeaIndexer::create(&config, Arc::clone(&operation_tx), Arc::clone(&store)).await?;

    tokio::spawn(async move {
        if let Err(e) = indexer.start(latest_processed_block).await {
            log::error!("Error while running indexer: {}", e);
        }
    });

    // -------------------- Start API --------------------
    log::info!("Starting API...");
    tokio::spawn(async move {
        if let Err(e) = api::start(AppState {
            order_repository: Arc::clone(&order_repository),
            trade_repository: Arc::clone(&trade_repository),
        })
        .await
        {
            log::error!("Error while running API: {}", e);
        }
    });
    // ---------------------------------------------------

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigint.recv() => log::info!("Received signal SIGINT. Shutting down."),
        _ = sigterm.recv() => log::info!("Received signal SIGTERM. Shutting down."),
    }

    Ok(())
}
