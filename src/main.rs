use dotenv::dotenv;
use error::Error;
use std::sync::Arc;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{mpsc::unbounded_channel, Mutex},
};

use crate::{
    config::Config,
    dispatcher::{OperationDispatcher, OperationMessage},
    pangea::PangeaIndexer,
    repo::{order, state},
    store::Store,
};

mod api;
mod config;
mod db;
mod dispatcher;
mod error;
mod pangea;
mod repo;
mod rpc;
mod store;
mod types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();

    let config = Config::load("config.mainnet.json")?;

    let db = db::build_connection().await?;

    let latest_processed_block = state::Query::find_latest_processed_block(&db)
        .await?
        .unwrap_or(config.pangea_start_block);
    let orders = order::Query::find(&db, 10_000, 0).await?;
    log::debug!("Orders: {:?}", orders.len());

    // Local store for order and trades
    let store = Arc::new(Mutex::new(Store::new(orders)));
    let db = Arc::new(db);

    // ------------------ Start dispatcher ------------------
    let mut operation_dispatcher = OperationDispatcher::new(Arc::clone(&db));
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

    // ------------------ Start RPC server ------------------
    log::info!("Starting RPC server...");
    tokio::spawn(rpc::serve(Arc::clone(&db)));

    // -------------------- Start API --------------------
    log::info!("Starting API...");
    tokio::spawn(api::serve(Arc::clone(&db)));
    // ---------------------------------------------------

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigint.recv() => log::info!("Received signal SIGINT. Shutting down."),
        _ = sigterm.recv() => log::info!("Received signal SIGTERM. Shutting down."),
    }

    Ok(())
}
