use utoipa::OpenApi;

use super::{order, trade};

#[derive(OpenApi)]
#[openapi(paths(
    order::list_orders,
    order::spread,
    order::best_bid,
    order::best_ask,
    trade::list_trades
))]
pub struct ApiDoc;
