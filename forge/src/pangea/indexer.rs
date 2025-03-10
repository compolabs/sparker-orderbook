use ethers_core::types::H256;
use fuels::accounts::provider::Provider;
use futures::StreamExt;
use pangea_client::{
    provider::FuelProvider, query::Bound, requests::fuel::GetSparkOrderRequest, ChainId, Client,
    ClientBuilder, Format, WsProvider,
};
use std::{collections::HashSet, env, str::FromStr};
use tokio::time::{sleep, Duration};

use crate::{
    dispatcher::{Operation, Update},
    error::Error,
    pangea::event::PangeaEvent,
    types::Sender,
};

const BATCH_SIZE: u64 = 100_000;

pub struct PangeaIndexer {
    pangea_host: String,
    provider: Provider,
    operation_tx: Sender<Operation>,
    chain_id: ChainId,
    market_id: String,
    market_name: String,
}

impl PangeaIndexer {
    pub async fn create(
        host: &str,
        market_id: &str,
        market_name: &str,
        operation_tx: Sender<Operation>,
    ) -> Result<Self, Error> {
        let chain_id = match env::var("CHAIN_ID").unwrap().as_str() {
            "FUEL" => ChainId::FUEL,
            _ => ChainId::FUELTESTNET,
        };

        let provider_url = match chain_id {
            ChainId::FUEL => Ok("mainnet.fuel.network"),
            ChainId::FUELTESTNET => Ok("testnet.fuel.network"),
            _ => Err(Error::InvalidChainId),
        }?;
        let provider = Provider::connect(provider_url).await?;

        Ok(Self {
            pangea_host: host.to_string(),
            provider,
            operation_tx,
            chain_id,
            market_id: market_id.to_string(),
            market_name: market_name.to_string(),
        })
    }

    pub async fn create_pangea_client(&self) -> Result<Client<WsProvider>, Error> {
        let username = env::var("PANGEA_USERNAME").unwrap();
        let password = env::var("PANGEA_PASSWORD").unwrap();

        let pangea_client = ClientBuilder::default()
            .endpoint(&self.pangea_host)
            .credential(username, password)
            .build::<WsProvider>()
            .await?;

        Ok(pangea_client)
    }

    pub async fn start(&self, latest_processed_block: i64) -> Result<(), Error> {
        // Get latest block number from blockchain
        let latest_block = self.provider.latest_block_height().await.unwrap() as i64;

        self.prune(latest_processed_block).await?;
        let latest_processed_block = self.catch_up(latest_processed_block, latest_block).await?;

        log::info!("[{}] LISTEN EVENTS FROM BLOCK: {}", self.market_name, latest_processed_block);
        self.listen_events(latest_processed_block).await?;

        Ok(())
    }

    pub async fn prune(&self, latest_processed_block: i64) -> Result<(), Error> {
        self.operation_tx
            .send(Operation::Prune(latest_processed_block))
            .unwrap();

        Ok(())
    }

    /// Catches up the processing of blocks from the latest processed block to the latest block
    /// from blockchain.
    ///
    /// # Arguments
    ///
    /// * `latest_processed_block` - Latest processed block number.
    /// * `to_block` - The block number until which to fetch historical events.
    ///
    /// # Returns
    ///
    /// Returns the block number of the latest processed block after catching up.
    ///
    pub async fn catch_up(
        &self,
        mut latest_processed_block: i64,
        to_block: i64,
    ) -> Result<i64, Error> {
        let client = self.create_pangea_client().await?;

        while latest_processed_block < to_block {
            let to_block = (latest_processed_block + BATCH_SIZE as i64).min(to_block);

            let contract_h256 = H256::from_str(&self.market_id)?;
            let batch_request = GetSparkOrderRequest {
                from_block: Bound::Exact(latest_processed_block),
                to_block: Bound::Exact(to_block),
                market_id__in: HashSet::from([contract_h256]),
                chains: HashSet::from([self.chain_id]),
                ..Default::default()
            };

            let stream = client
                .get_fuel_spark_orders_by_format(batch_request, Format::JsonStream, false)
                .await
                .expect("Failed to get fuel spark orders batch");
            futures::pin_mut!(stream);

            while let Some(data) = stream.next().await {
                match data {
                    Ok(data) => {
                        let data = String::from_utf8(data)?;
                        let event = serde_json::from_str::<PangeaEvent>(&data)?;
                        latest_processed_block = event.block_number;

                        // Process event with collecting operations to dispatch
                        self.handle_event(&event).await;
                    }
                    Err(e) => {
                        log::error!("Error in the stream of historical events: {e}");
                        break;
                    }
                }
            }

            // Dispatch operations
            self.operation_tx
                .send(Operation::Dispatch(latest_processed_block))
                .unwrap();

            log::debug!(
                "[{}] PROCESSED: {}",
                self.market_name,
                latest_processed_block
            );
            latest_processed_block = to_block;
        }

        Ok(latest_processed_block)
    }

    /// Listens for new events and processes them in real-time.
    ///
    /// If an error occurs while processing the stream of events, it logs the error and attempts to reconnect
    /// after a short delay.
    ///
    /// # Arguments
    ///
    /// * `latest_processed_block` - The block number of the latest processed block.
    ///
    async fn listen_events(&self, mut latest_processed_block: i64) -> Result<(), Error> {
        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(32);

        loop {
            match self.create_pangea_client().await {
                Ok(client) => {
                    let contract_h256 = H256::from_str(&self.market_id)?;
                    let deltas_request = GetSparkOrderRequest {
                        from_block: Bound::Exact(latest_processed_block + 1),
                        to_block: Bound::Subscribe,
                        market_id__in: HashSet::from([contract_h256]),
                        chains: HashSet::from([self.chain_id]),
                        ..Default::default()
                    };

                    match client
                        .get_fuel_spark_orders_by_format(deltas_request, Format::JsonStream, true)
                        .await
                    {
                        Ok(stream) => {
                            backoff = Duration::from_secs(1);
                            futures::pin_mut!(stream);

                            while let Some(data) = stream.next().await {
                                match data {
                                    Ok(data) => {
                                        let data = String::from_utf8(data)?;
                                        let event = serde_json::from_str::<PangeaEvent>(&data)?;
                                        latest_processed_block = event.block_number;

                                        log::debug!(
                                            "[{}] LATEST_PROCESSED_BLOCK: {}",
                                            self.market_name,
                                            latest_processed_block
                                        );

                                        self.handle_event(&event).await;
                                        self.operation_tx
                                            .send(Operation::Dispatch(latest_processed_block))
                                            .unwrap();
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "Error in the stream of new orders (deltas): {e}"
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to get fuel spark deltas: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create pangea client: {e}");
                }
            }

            log::debug!("[{}] RECONNECTING...", self.market_name);
            sleep(backoff).await;
            backoff = (backoff * 2).min(max_backoff);
        }
    }

    /// Handles a Pangea event by dispatching the appropriate operation.
    ///
    /// # Arguments
    ///
    /// * `event` - A reference to the `PangeaEvent` to be handled.
    ///
    /// # Errors
    ///
    /// Logs an error if the event type is unknown.
    pub async fn handle_event(&self, event: &PangeaEvent) {
        if let Some(event_type) = event.event_type.as_deref() {
            match event_type {
                "Open" => {
                    if let Some(order) = event.build_order() {
                        self.operation_tx
                            .send(Operation::Update(Update::OpenOrder(order)))
                            .unwrap();
                    }
                }
                "Trade" => {
                    if let Some(trade) = event.build_trade() {
                        self.operation_tx
                            .send(Operation::Update(Update::Trade(trade)))
                            .unwrap();
                    }
                }
                "Cancel" => {
                    self.operation_tx
                        .send(Operation::Update(Update::CancelOrder(
                            event.order_id.clone(),
                        )))
                        .unwrap();
                }
                _ => {
                    log::error!("UNKNOWN_EVENT_TYPE: {}", event_type);
                }
            }
        }
    }
}
