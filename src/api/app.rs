use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    error::Error,
    models::{Order, OrderType, Trade},
    repos::{OrderRepository, TradeRepository},
};

#[derive(OpenApi)]
#[openapi(paths(list_orders, spread, best_bid, best_ask, list_trades))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub order_repository: Arc<OrderRepository>,
    pub trade_repository: Arc<TradeRepository>,
}

pub async fn start(state: AppState) -> Result<(), Error> {
    let app = Router::new()
        .route("/orders/list", get(list_orders))
        .route("/orders/spread", get(spread))
        .route("/orders/best-bid", get(best_bid))
        .route("/orders/best-ask", get(best_ask))
        .route("/trades/list", get(list_trades))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Serialize, Deserialize, ToSchema)]
struct Spread {
    pub best_bid: Option<Order>,
    pub best_ask: Option<Order>,
}

#[utoipa::path(
    get,
    path = "/orders/spread",
    responses(
        (status = 200, description = "Returns spread as two orders: best bid and best ask", body = Spread)
    )
)]
async fn spread(
    State(AppState {
        order_repository, ..
    }): State<AppState>,
) -> Result<Json<Spread>, (StatusCode, String)> {
    let best_bid = order_repository.best_bid().await.map_err(internal_error)?;
    let best_ask = order_repository.best_ask().await.map_err(internal_error)?;

    Ok(Json(Spread { best_bid, best_ask }))
}

#[utoipa::path(
    get,
    path = "/orders/best-bid",
    responses(
        (status = 200, description = "Returns best bid order", body = Order)
    )
)]
async fn best_bid(
    State(AppState {
        order_repository, ..
    }): State<AppState>,
) -> Result<Json<Order>, (StatusCode, String)> {
    let res = order_repository.best_bid().await.map_err(internal_error)?;
    Ok(Json(res.unwrap()))
}

#[utoipa::path(
    get,
    path = "/orders/best-ask",
    responses(
        (status = 200, description = "Returns best ask order", body = Order)
    )
)]
async fn best_ask(
    State(AppState {
        order_repository, ..
    }): State<AppState>,
) -> Result<Json<Order>, (StatusCode, String)> {
    let res = order_repository.best_ask().await.map_err(internal_error)?;
    Ok(Json(res.unwrap()))
}

#[derive(Deserialize, IntoParams)]
struct ListOrdersParams {
    order_type: Option<OrderType>,
    limit: Option<u64>,
    offset: Option<u64>,
}

#[utoipa::path(
    get,
    path = "/orders/list",
    params(
        ListOrdersParams,
    ),
    responses(
        (status = 200, description = "Returns list of orders", body = Vec<Order>)
    )
)]
async fn list_orders(
    Query(ListOrdersParams {
        order_type,
        limit,
        offset,
    }): Query<ListOrdersParams>,
    State(AppState {
        order_repository, ..
    }): State<AppState>,
) -> Result<Json<Vec<Order>>, (StatusCode, String)> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let res = match order_type {
        Some(order_type) => {
            order_repository
                .orders_by_type(order_type, limit, offset)
                .await
        }
        None => order_repository.orders(limit, offset).await,
    }
    .map_err(internal_error)?;

    Ok(Json(res))
}

#[derive(Deserialize, IntoParams)]
struct ListTradesParams {
    limit: Option<u64>,
    offset: Option<u64>,
}

#[utoipa::path(
    get,
    path = "/trades/list",
    params(
        ListTradesParams,
    ),
    responses(
        (status = 200, description = "Returns list of trades", body = Vec<Trade>)
    )
)]
async fn list_trades(
    Query(ListTradesParams { limit, offset }): Query<ListTradesParams>,
    State(AppState {
        trade_repository, ..
    }): State<AppState>,
) -> Result<Json<Vec<Trade>>, (StatusCode, String)> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let res = trade_repository
        .trades(limit, offset)
        .await
        .map_err(internal_error)?;

    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
