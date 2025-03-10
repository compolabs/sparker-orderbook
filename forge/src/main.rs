use dispatcher::Operation;
use dotenv::dotenv;
use error::Error;
use sparker_core::repo::state;
use std::sync::Arc;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{mpsc::unbounded_channel, Mutex},
};

use crate::{config::Config, dispatcher::OperationDispatcher, pangea::PangeaIndexer};

mod config;
mod db;
mod dispatcher;
mod error;
mod pangea;
mod types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();

    let config = Config::load("config.mainnet.json")?;

    let db_conn = db::build_connection().await?;
    let db_conn = Arc::new(db_conn);

    // ------------------ Start indexers ------------------
    log::info!("Starting indexers...");
    for market in config.markets {
        // -------------- Start operation dispatcher --------------
        let (operation_tx, operation_rx) = unbounded_channel::<Operation>();
        let operation_tx = Arc::new(operation_tx);
        let operation_rx = Arc::new(Mutex::new(operation_rx));

        let operation_dispatcher = OperationDispatcher::new(
            market.id.clone(),
            Arc::clone(&db_conn),
            Arc::clone(&operation_rx),
        );
        tokio::spawn(async move {
            operation_dispatcher.start().await;
        });

        let indexer = PangeaIndexer::create(
            &config.pangea_host,
            &market.id,
            &market.name,
            Arc::clone(&operation_tx),
        )
        .await?;

        // Get the latest processed block from the database
        let latest_processed_block =
            state::Query::find_latest_processed_block(&db_conn, &market.id)
                .await?
                .unwrap_or(config.pangea_start_block);

        tokio::spawn(async move {
            if let Err(e) = indexer.start(latest_processed_block).await {
                log::error!("Error while running indexer: {}", e);
            }
        });
    }
    // ---------------------------------------------------

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigint.recv() => log::info!("Received signal SIGINT. Shutting down."),
        _ = sigterm.recv() => log::info!("Received signal SIGTERM. Shutting down."),
    }

    Ok(())
}
