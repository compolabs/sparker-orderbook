use axum::{http::StatusCode, routing::get, Router};
use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use std::{net::SocketAddr, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    openapi::ApiDoc,
    order::{best_ask, best_bid, list_orders, spread},
    trade::list_trades,
};

mod db;
mod openapi;
mod order;
mod trade;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let db = db::build_connection()
        .await
        .expect("Failed to connect to database");
    let db = Arc::new(db);

    serve(db).await;
}

pub async fn serve(db: Arc<DatabaseConnection>) {
    let app = Router::new()
        .route("/orders/list", get(list_orders))
        .route("/orders/spread", get(spread))
        .route("/orders/best-bid", get(best_bid))
        .route("/orders/best-ask", get(best_ask))
        .route("/trades/list", get(list_trades))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(AppState { db });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Failed to serve");
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
