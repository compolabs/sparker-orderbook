use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use sparker_core::repo::{order, trade};
use sparker_proto::{
    api::{
        orderbook_server::{Orderbook, OrderbookServer},
        OrderRequest, OrderResponse, OrdersRequest, OrdersResponse, SpreadRequest, SpreadResponse,
        TradeRequest, TradeResponse, TradesRequest, TradesResponse,
    },
    types as proto,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

mod db;

pub struct RpcServer {
    db_conn: Arc<DatabaseConnection>,
}

#[tonic::async_trait]
impl Orderbook for RpcServer {
    async fn list_orders(
        &self,
        request: Request<OrdersRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let request = request.into_inner();
        let limit = request.limit;
        let order_type = proto::OrderType::from_repr(request.order_type);
        let user_ne = request.user_ne;

        let orders = match order_type {
            Some(order_type) => {
                order::Query::find_by_type(&self.db_conn, order_type.into(), limit, 0, user_ne)
                    .await
            }
            None => order::Query::find(&self.db_conn, limit, 0).await,
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
        let user = request.user;
        // let mut events_rx = self.events.subscribe();

        let (tx, rx) = mpsc::channel(4);

        // tokio::spawn(async move {
        //     while let Ok(event) = events_rx.recv().await {
        //         match event {
        //             // Only care about user order updates if user is Some
        //             Event::OrderUpdated(order)
        //                 if user.as_ref().map_or(true, |user| &order.user == user) =>
        //             {
        //                 let response = OrderResponse {
        //                     order: Some(order.into()),
        //                 };
        //                 let _ = tx.send(Ok(response)).await;
        //             }
        //             _ => {}
        //         }
        //     }
        // });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn spread(
        &self,
        request: Request<SpreadRequest>,
    ) -> Result<Response<SpreadResponse>, Status> {
        let request = request.into_inner();
        let user_ne = request.user_ne;
        let best_bid = order::Query::find_best_bid(&self.db_conn, user_ne.clone())
            .await
            .unwrap()
            .map(|o| o.into());
        let best_ask = order::Query::find_best_ask(&self.db_conn, user_ne)
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
        let limit = request.limit;

        let trades = trade::Query::find(&self.db_conn, limit, 0).await.unwrap();
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

pub async fn serve(db_conn: Arc<DatabaseConnection>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 50051));

    if let Err(e) = Server::builder()
        .add_service(OrderbookServer::new(RpcServer { db_conn }))
        .serve(addr)
        .await
    {
        log::error!("Failed to serve: {}", e);
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

    log::info!("Starting RPC server...");
    serve(db_conn).await;
}
