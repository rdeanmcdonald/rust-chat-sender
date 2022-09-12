use axum::http::StatusCode;
use axum::{response::Json, routing::post, Router};
use futures::future::join_all;
use message::ClientMessage;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};
use std::boxed::Box;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::spawn;
use tokio::task::JoinHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::{uuid, Uuid};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            // example log str to get trace logs for kafka client:
            // RUST_LOG="librdkafka=trace,rdkafka::client=debug"
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_handle = spawn(async move {
        // kafka producer is cheap to clone
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:29092")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create producer");

        let app = Router::new().route("/send", post(move |body| send(body, producer.clone())));

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::debug!("listening on {}", addr);

        let _result = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await;
    });

    let handles = vec![server_handle];
    let _results = join_all(handles).await;
}

async fn send(Json(client_message): Json<ClientMessage>, producer: FutureProducer) -> Json<Value> {
    tracing::debug!("Received {:?}", client_message);

    let a = client_message.to.into_bytes();
    let delivery_status = producer
        .send(
            FutureRecord::to("messages")
                .payload(&client_message.content)
                .key(&client_message.to.into_bytes()),
            Duration::from_secs(0),
        )
        .await;

    tracing::debug!("Delivery status for message {:?} received", delivery_status);

    Json(json!({ "data": 42 }))
}
