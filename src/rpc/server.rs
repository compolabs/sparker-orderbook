use sea_orm::DatabaseConnection;
use sparker_core::repo::{order, trade};
use sparker_rpc::proto::{
    self,
    orderbook_server::{Orderbook, OrderbookServer},
    ListOrdersRequest, ListOrdersResponse, ListTradesRequest, ListTradesResponse,
    ListUserOrdersRequest, SpreadRequest, SpreadResponse,
};
use std::{net::SocketAddr, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};

use crate::error::Error;

pub struct RpcServer {
    db: Arc<DatabaseConnection>,
}

#[tonic::async_trait]
impl Orderbook for RpcServer {
    async fn list_orders(
        &self,
        request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        let request = request.into_inner();
        let limit = request.limit;
        let order_type = proto::OrderType::from_repr(request.order_type);
        let user_ne = request.user_ne;

        let orders = match order_type {
            Some(order_type) => {
                order::Query::find_by_type(&self.db, order_type.into(), limit, 0, user_ne).await
            }
            None => order::Query::find(&self.db, limit, 0).await,
        }
        .unwrap();

        let orders = orders
            .into_iter()
            .map(|order| order.into())
            .collect::<Vec<proto::Order>>();

        let response = ListOrdersResponse { orders };
        Ok(Response::new(response))
    }

    async fn list_user_orders(
        &self,
        request: Request<ListUserOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        let request = request.into_inner();
        let limit = request.limit;
        let user = request.user;

        let orders = order::Query::find_by_user(&self.db, user, limit, 0)
            .await
            .unwrap();
        let orders = orders
            .into_iter()
            .map(|order| order.into())
            .collect::<Vec<proto::Order>>();

        let response = ListOrdersResponse { orders };
        Ok(Response::new(response))
    }

    async fn spread(
        &self,
        request: Request<SpreadRequest>,
    ) -> Result<Response<SpreadResponse>, Status> {
        let request = request.into_inner();
        let user_ne = request.user_ne;
        let best_bid = order::Query::find_best_bid(&self.db, user_ne.clone())
            .await
            .unwrap()
            .map(|o| o.into());
        let best_ask = order::Query::find_best_ask(&self.db, user_ne)
            .await
            .unwrap()
            .map(|o| o.into());

        let response = proto::SpreadResponse { best_bid, best_ask };
        Ok(Response::new(response))
    }

    async fn list_trades(
        &self,
        request: Request<ListTradesRequest>,
    ) -> Result<Response<ListTradesResponse>, Status> {
        let request = request.into_inner();
        let limit = request.limit;

        let trades = trade::Query::find(&self.db, limit, 0).await.unwrap();
        let trades = trades
            .into_iter()
            .map(|trade| trade.into())
            .collect::<Vec<proto::Trade>>();

        let response = ListTradesResponse { trades };
        Ok(Response::new(response))
    }
}

pub async fn serve(db: Arc<DatabaseConnection>) -> Result<(), Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 50051));

    let rpc_server = RpcServer { db };
    Server::builder()
        .add_service(OrderbookServer::new(rpc_server))
        .serve(addr)
        .await?;

    Ok(())
}
