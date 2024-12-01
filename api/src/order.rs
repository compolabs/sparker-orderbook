use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sparker_core::{repo::order, Order, OrderType};
use utoipa::{IntoParams, ToSchema};

use crate::{internal_error, AppState};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Spread {
    pub best_bid: Option<Order>,
    pub best_ask: Option<Order>,
}

#[derive(Deserialize, IntoParams)]
pub struct SpreadParams {
    market_id: String,
    user_ne: Option<String>,
}

#[utoipa::path(
    get,
    path = "/orders/spread",
    params(
        SpreadParams,
    ),
    responses(
        (status = 200, description = "Returns spread as two orders: best bid and best ask", body = Spread)
    )
)]
pub async fn spread(
    Query(SpreadParams { market_id, user_ne }): Query<SpreadParams>,
    State(AppState { db_conn, .. }): State<AppState>,
) -> Result<Json<Spread>, (StatusCode, String)> {
    let best_bid = order::Query::find_best_bid(&db_conn, market_id.clone(), user_ne.clone())
        .await
        .map_err(internal_error)?;
    let best_ask = order::Query::find_best_ask(&db_conn, market_id, user_ne)
        .await
        .map_err(internal_error)?;

    Ok(Json(Spread { best_bid, best_ask }))
}

#[derive(Deserialize, IntoParams)]
pub struct BestOrderParams {
    market_id: String,
    user_ne: Option<String>,
}

#[utoipa::path(
    get,
    path = "/orders/best-bid",
    params(
        BestOrderParams,
    ),
    responses(
        (status = 200, description = "Returns best bid order", body = Order)
    )
)]
pub async fn best_bid(
    Query(BestOrderParams { market_id, user_ne }): Query<BestOrderParams>,
    State(AppState { db_conn, .. }): State<AppState>,
) -> Result<Json<Option<Order>>, (StatusCode, String)> {
    let res = order::Query::find_best_bid(&db_conn, market_id, user_ne)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}

#[utoipa::path(
    get,
    path = "/orders/best-ask",
    params(
        BestOrderParams,
    ),
    responses(
        (status = 200, description = "Returns best ask order", body = Order)
    )
)]
pub async fn best_ask(
    Query(BestOrderParams { market_id, user_ne }): Query<BestOrderParams>,
    State(AppState { db_conn, .. }): State<AppState>,
) -> Result<Json<Option<Order>>, (StatusCode, String)> {
    let res = order::Query::find_best_ask(&db_conn, market_id, user_ne)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}

#[derive(Deserialize, IntoParams)]
pub struct ListOrdersParams {
    market_id: String,
    order_type: Option<OrderType>,
    limit: Option<u64>,
    offset: Option<u64>,
    user_ne: Option<String>,
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
pub async fn list_orders(
    Query(ListOrdersParams {
        market_id,
        order_type,
        limit,
        offset,
        user_ne,
    }): Query<ListOrdersParams>,
    State(AppState { db_conn, .. }): State<AppState>,
) -> Result<Json<Vec<Order>>, (StatusCode, String)> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let res = match order_type {
        Some(order_type) => {
            order::Query::find_by_type(&db_conn, market_id, order_type, limit, offset, user_ne)
                .await
        }
        None => order::Query::find(&db_conn, market_id, limit, offset).await,
    }
    .map_err(internal_error)?;

    Ok(Json(res))
}
