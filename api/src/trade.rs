use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sparker_core::{repo::trade, Trade};
use utoipa::IntoParams;

use crate::{internal_error, AppState};

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
    State(AppState { db_conn, .. }): State<AppState>,
) -> Result<Json<Vec<Trade>>, (StatusCode, String)> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let res = trade::Query::find(&db_conn, limit, offset)
        .await
        .map_err(internal_error)?;

    Ok(Json(res))
}
