use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sparker_core::Trade;
use utoipa::IntoParams;

use super::api::{internal_error, AppState};
use crate::repo::trade;

#[derive(Deserialize, IntoParams)]
pub struct ListTradesParams {
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
pub async fn list_trades(
    Query(ListTradesParams { limit, offset }): Query<ListTradesParams>,
    State(AppState { db, .. }): State<AppState>,
) -> Result<Json<Vec<Trade>>, (StatusCode, String)> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let res = trade::Query::find(&db, limit, offset)
        .await
        .map_err(internal_error)?;

    Ok(Json(res))
}
