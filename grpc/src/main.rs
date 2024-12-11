use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use sparker_core::{
    repo::{order, trade},
    Order,
};
use sparker_proto::{
    api::{
        orderbook_server::{Orderbook, OrderbookServer},
        OrderRequest, OrderResponse, OrdersRequest, OrdersResponse, SpreadRequest, SpreadResponse,
        TradeRequest, TradeResponse, TradesRequest, TradesResponse,
    },
    types as proto,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::event::Event;

mod db;
mod event;

pub struct RpcServer {
    db_conn: Arc<DatabaseConnection>,
    events_tx: broadcast::Sender<Event>,
}

#[tonic::async_trait]
impl Orderbook for RpcServer {
    async fn list_orders(
        &self,
        request: Request<OrdersRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let request = request.into_inner();
        let market_id = request.market_id;
        let limit = request.limit;
        let order_type = proto::OrderType::from_repr(request.order_type);
        let user_ne = request.user_ne;

        let orders = match order_type {
            Some(order_type) => {
                order::Query::find_by_type(
                    &self.db_conn,
                    market_id,
                    order_type.into(),
                    limit,
                    0,
                    user_ne,
                )
                .await
            }
            None => order::Query::find(&self.db_conn, market_id, limit, 0).await,
        }
        .unwrap();

        let orders = orders
            .into_iter()
            .map(|order| order.into())
            .collect::<Vec<proto::Order>>();

        let response = OrdersResponse { orders };
        Ok(Response::new(response))
    }

    type SubscribeOrderUpdatesStream = ReceiverStream<Result<OrderResponse, Status>>;
    async fn subscribe_order_updates(
        &self,
        request: Request<OrderRequest>,
    ) -> Result<Response<Self::SubscribeOrderUpdatesStream>, Status> {
        let request = request.into_inner();
        let market_id = request.market_id;
        let user = request.user;
        let mut events_rx = self.events_tx.subscribe();

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            while let Ok(event) = events_rx.recv().await {
                match event {
                    // Only care about user order updates
                    Event::OrderUpdate(order)
                        if user.as_ref().map_or(true, |user| &order.user == user)
                            && order.market_id == market_id =>
                    {
                        let response = OrderResponse {
                            order: Some(order.into()),
                        };
                        let _ = tx.send(Ok(response)).await;
                    }
                    _ => {}
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn spread(
        &self,
        request: Request<SpreadRequest>,
    ) -> Result<Response<SpreadResponse>, Status> {
        let request = request.into_inner();
        let market_id = request.market_id;
        let user_ne = request.user_ne;
        let best_bid =
            order::Query::find_best_bid(&self.db_conn, market_id.clone(), user_ne.clone())
                .await
                .unwrap()
                .map(|o| o.into());
        let best_ask = order::Query::find_best_ask(&self.db_conn, market_id, user_ne)
            .await
            .unwrap()
            .map(|o| o.into());

        let response = SpreadResponse { best_bid, best_ask };
        Ok(Response::new(response))
    }

    async fn list_trades(
        &self,
        request: Request<TradesRequest>,
    ) -> Result<Response<TradesResponse>, Status> {
        let request = request.into_inner();
        let market_id = request.market_id;
        let limit = request.limit;

        let trades = trade::Query::find(&self.db_conn, market_id, limit, 0)
            .await
            .unwrap();
        let trades = trades
            .into_iter()
            .map(|trade| trade.into())
            .collect::<Vec<proto::Trade>>();

        let response = TradesResponse { trades };
        Ok(Response::new(response))
    }

    type SubscribeTradesStream = ReceiverStream<Result<TradeResponse, Status>>;
    async fn subscribe_trades(
        &self,
        request: Request<TradeRequest>,
    ) -> Result<Response<Self::SubscribeTradesStream>, Status> {
        let request = request.into_inner();
        let market_id = request.market_id;
        let user = request.user;
        // let mut events_rx = self.events.subscribe();

        let (tx, rx) = mpsc::channel(4);

        // tokio::spawn(async move {
        //     while let Ok(event) = events_rx.recv().await {
        //         match event {
        //             Event::Traded(trade)
        //                 if user.as_ref().map_or(true, |user| &trade.user == user) =>
        //             {
        //                 let response = TradeResponse {
        //                     trade: Some(trade.into()),
        //                 };
        //                 let _ = tx.send(Ok(response)).await;
        //             }
        //             _ => {}
        //         }
        //     }
        // });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

async fn serve(db_conn: Arc<DatabaseConnection>, events_tx: broadcast::Sender<Event>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 50051));

    if let Err(e) = Server::builder()
        .add_service(OrderbookServer::new(RpcServer { db_conn, events_tx }))
        .serve(addr)
        .await
    {
        log::error!("Failed to serve: {}", e);
    }
}

#[allow(clippy::single_match)]
async fn listen_updates(db_conn: Arc<DatabaseConnection>, events_tx: broadcast::Sender<Event>) {
    let mut listener = db::build_listener(&db_conn).await.unwrap();
    listener.listen("order_updates").await.unwrap();

    while let Ok(notification) = listener.recv().await {
        match notification.channel() {
            "order_updates" => match Order::from_payload(notification.payload()) {
                Ok(order) => {
                    if let Err(e) = events_tx.send(Event::OrderUpdate(order)) {
                        log::error!("SEND_ORDER_UPDATE_ERROR: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("PARSE_ORDER_ERROR: {}", e);
                }
            },
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let db_conn = db::build_connection()
        .await
        .expect("Failed to connect to database");
    let db_conn = Arc::new(db_conn);

    let (events_tx, _) = broadcast::channel::<Event>(100);
    tokio::spawn(listen_updates(db_conn.clone(), events_tx.clone()));

    log::info!("Starting gRPC server...");
    serve(db_conn, events_tx).await;
}
