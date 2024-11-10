use ethers_core::types::H256;
use fuels::accounts::provider::Provider;
use futures::StreamExt;
use pangea_client::{
    provider::FuelProvider, query::Bound, requests::fuel::GetSparkOrderRequest, ChainId, Client,
    ClientBuilder, Format, WsProvider,
};
use sparker_core::{LimitType, OrderStatus, UpdateOrder};
use std::{collections::HashSet, env, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    config::Config,
    dispatcher::{Operation, OperationMessage},
    error::Error,
    pangea::event::PangeaEvent,
    store::Store,
    types::Sender,
};

const BATCH_SIZE: u64 = 10_000;

pub struct PangeaIndexer {
    pangea_client: Client<WsProvider>,
    fuel_provider: Provider,
    operation_tx: Sender<OperationMessage>,
    store: Arc<Mutex<Store>>,
    chain_id: ChainId,
    contract_h256: H256,
}

impl PangeaIndexer {
    pub async fn create(
        config: &Config,
        operation_tx: Sender<OperationMessage>,
        store: Arc<Mutex<Store>>,
    ) -> Result<Self, Error> {
        let chain_id = match env::var("CHAIN_ID").unwrap().as_str() {
            "FUEL" => ChainId::FUEL,
            _ => ChainId::FUELTESTNET,
        };

        let username = env::var("PANGEA_USERNAME").unwrap();
        let password = env::var("PANGEA_PASSWORD").unwrap();

        let pangea_client = ClientBuilder::default()
            .endpoint(&config.pangea_host)
            .credential(username, password)
            .build::<WsProvider>()
            .await?;

        let provider_url = match chain_id {
            ChainId::FUEL => Ok("mainnet.fuel.network"),
            ChainId::FUELTESTNET => Ok("testnet.fuel.network"),
            _ => Err(Error::InvalidChainId),
        }?;
        let fuel_provider = Provider::connect(provider_url).await?;

        log::info!("CHAIN: {:?}, PROVIDER: {:?}", chain_id, provider_url);

        Ok(Self {
            pangea_client,
            fuel_provider,
            operation_tx,
            store,
            chain_id,
            contract_h256: H256::from_str(&config.market_id.to_string())?,
        })
    }

    pub async fn start(&self, latest_processed_block: i64) -> Result<(), Error> {
        let latest_block = self.fuel_provider.latest_block_height().await.unwrap() as i64;

        log::debug!("LATEST_PROCESSED_BLOCK: {}", latest_processed_block);

        log::info!("Fetch historical events, until block: {}", latest_block);
        let latest_processed_block = self
            .fetch_historical_events(latest_processed_block, latest_block)
            .await?;

        log::info!("Listen events, from block: {}", latest_processed_block);
        self.listen_events(latest_processed_block).await?;

        Ok(())
    }

    pub async fn fetch_historical_events(
        &self,
        mut latest_processed_block: i64,
        latest_block: i64,
    ) -> Result<i64, Error> {
        while latest_processed_block < latest_block {
            let to_block = (latest_processed_block + BATCH_SIZE as i64).min(latest_block);

            let batch_request = GetSparkOrderRequest {
                from_block: Bound::Exact(latest_processed_block),
                to_block: Bound::Exact(to_block),
                market_id__in: HashSet::from([self.contract_h256]),
                chains: HashSet::from([self.chain_id]),
                ..Default::default()
            };

            let stream = self
                .pangea_client
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
                        self.process_event(&event).await;
                    }
                    Err(e) => {
                        log::error!("Error in the stream of historical events: {e}");
                        break;
                    }
                }
            }

            // Dispatch operations
            self.operation_tx.send(OperationMessage::Dispatch).unwrap();

            log::debug!("PROCESSED: {}", latest_processed_block);
            latest_processed_block = to_block;
        }

        self.operation_tx
            .send(OperationMessage::SetLatestProcessedBlock(
                latest_processed_block,
            ))
            .unwrap();

        Ok(latest_processed_block)
    }

    async fn listen_events(&self, mut latest_processed_block: i64) -> Result<(), Error> {
        loop {
            let deltas_request = GetSparkOrderRequest {
                from_block: Bound::Exact(latest_processed_block + 1),
                to_block: Bound::Subscribe,
                market_id__in: HashSet::from([self.contract_h256]),
                chains: HashSet::from([self.chain_id]),
                ..Default::default()
            };

            let stream = self
                .pangea_client
                .get_fuel_spark_orders_by_format(deltas_request, Format::JsonStream, true)
                .await
                .expect("Failed to get fuel spark deltas");
            futures::pin_mut!(stream);

            log::debug!("STREAM");

            while let Some(data) = stream.next().await {
                match data {
                    Ok(data) => {
                        let data = String::from_utf8(data)?;
                        let event = serde_json::from_str::<PangeaEvent>(&data)?;
                        latest_processed_block = event.block_number;

                        log::debug!("LATEST_PROCESSED_BLOCK: {}", latest_processed_block);

                        self.process_event(&event).await;
                        self.operation_tx.send(OperationMessage::Dispatch).unwrap();

                        self.operation_tx
                            .send(OperationMessage::SetLatestProcessedBlock(
                                latest_processed_block,
                            ))
                            .unwrap();
                    }
                    Err(e) => {
                        log::error!("Error in the stream of new orders (deltas): {e}");
                        break;
                    }
                }
            }

            log::debug!("Reconnecting to listen for new deltas...");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    pub async fn process_event(&self, event: &PangeaEvent) {
        if let Some(event_type) = event.event_type.as_deref() {
            match event_type {
                "Open" => self.process_open(event).await,
                "Trade" => self.process_trade(event).await,
                "Cancel" => self.process_cancel(event).await,
                _ => {
                    log::error!("UNKNOWN_EVENT_TYPE: {}", event_type);
                }
            }
        }
    }

    pub async fn process_open(&self, event: &PangeaEvent) {
        if let Some(order) = event.build_order() {
            let mut store = self.store.lock().await;

            store.insert_order(order.clone());
            self.operation_tx
                .send(OperationMessage::Add(Operation::InsertOrder(order)))
                .unwrap();
        }
    }

    pub async fn process_trade(&self, event: &PangeaEvent) {
        if let Some(trade_size) = event.amount {
            let limit_type = event.limit_type().expect("INVALID_LIMIT_TYPE");

            let mut store = self.store.lock().await;
            let order = store.order(&event.order_id);

            match order {
                Some(order) => {
                    let trade_size = trade_size as u64;
                    let (status, amount) = match limit_type {
                        LimitType::GTC => {
                            if order.amount > trade_size {
                                (
                                    OrderStatus::PartiallyMatched,
                                    Some(order.amount - trade_size),
                                )
                            } else {
                                // Fully matched
                                (OrderStatus::Matched, None)
                            }
                        }
                        // FOK or IOC as fully matched
                        _ => (OrderStatus::Matched, None),
                    };

                    // Update order
                    store.update_order(UpdateOrder {
                        order_id: event.order_id.clone(),
                        amount,
                        status,
                    });
                    self.operation_tx
                        .send(OperationMessage::Add(Operation::UpdateOrder(UpdateOrder {
                            order_id: event.order_id.clone(),
                            amount,
                            status,
                        })))
                        .unwrap();

                    // Create new trade
                    if let Some(trade) = event.build_trade() {
                        store.insert_trade(trade.clone());
                        self.operation_tx
                            .send(OperationMessage::Add(Operation::InsertTrade(trade)))
                            .unwrap();
                    }
                }
                None => {
                    log::error!("ORDER_NOT_FOUND: {}", event.order_id);
                }
            }
        }
    }

    pub async fn process_cancel(&self, event: &PangeaEvent) {
        let mut store = self.store.lock().await;
        let order = store.order(&event.order_id);
        if order.is_some() {
            let updated_order = UpdateOrder {
                order_id: event.order_id.clone(),
                amount: None,
                status: OrderStatus::Cancelled,
            };

            store.update_order(updated_order.clone());
            self.operation_tx
                .send(OperationMessage::Add(Operation::UpdateOrder(updated_order)))
                .unwrap();
        }
    }
}
