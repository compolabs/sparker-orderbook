use axum::{http::StatusCode, routing::get, Router};
use dotenv::dotenv;
use migration::{Migrator, MigratorTrait};
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
    pub db_conn: Arc<DatabaseConnection>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let db_conn = db::build_connection()
        .await
        .expect("Failed to connect to database");
    Migrator::up(&db_conn, None)
        .await
        .expect("Failed to run migrations");
    let db_conn = Arc::new(db_conn);

    log::info!("Starting API server...");
    serve(AppState { db_conn }).await;
}

pub async fn serve(state: AppState) {
    let app = Router::new()
        .route("/orders/list", get(list_orders))
        .route("/orders/spread", get(spread))
        .route("/orders/best-bid", get(best_bid))
        .route("/orders/best-ask", get(best_ask))
        .route("/trades/list", get(list_trades))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3011));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Failed to serve: {}", e);
    }
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
