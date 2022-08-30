use axum::{response::Json, routing::get, Router};
use futures::future::join_all;
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::spawn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = Router::new().route("/send", get(json));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let server_handle = spawn(async move {
        // maybe later deal with errors in this task
        let _result = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await;
    });
    let handles = vec![server_handle];
    let _results = join_all(handles).await;
}

async fn json() -> Json<Value> {
    tracing::debug!("from method json {}", "hi");
    Json(json!({ "data": 42 }))
}
