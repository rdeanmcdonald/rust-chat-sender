use axum::{response::Json, routing::get, Router};
use futures::future::join_all;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::spawn;
use tokio::task::JoinHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// producer
use futures::future::Future;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::boxed::Box;

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

    // ########## SERVER ##########
    let server_handle = spawn(async move {
        // build our application with some routes
        let app = Router::new().route("/send", get(json));

        // run it with hyper
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::debug!("listening on {}", addr);

        // maybe later deal with errors in this task
        let _result = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await;
    });

    // let kafka_producer_handle = ...
    // follow instructions for this example:
    // https://github.com/fede1024/rust-rdkafka/blob/master/examples/simple_producer.rs
    let producer_handle = spawn(async move {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:29092")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create producer");

        let _result = produce(&producer).await;
    });

    let handles = vec![server_handle, producer_handle];
    let _results = join_all(handles).await;
}

async fn json() -> Json<Value> {
    tracing::debug!("from method json {}", "hi");
    Json(json!({ "data": 42 }))
}

async fn produce(producer: &FutureProducer) -> Result<(), Box<dyn std::error::Error>> {
    let _delivery_handles: Vec<JoinHandle<Result<(i32, i64), (KafkaError, OwnedMessage)>>> = (0..5)
        .map(|i| {
            // docs say it's cheap clone
            let p = producer.clone();
            spawn(async move {
                let value = &format!("My message {}", i);
                tracing::debug!("Preparing to produce record: {} {}", "alice", value);

                let delivery_status = p
                    .send(
                        FutureRecord::to("topic_testing")
                            .payload(value)
                            .key(&format!("My key {}", i)),
                        Duration::from_secs(0),
                    )
                    .await;

                // This will be executed when the result is received.
                tracing::info!("Delivery status for message {} received", i);
                delivery_status
            })
        })
        .collect();

    Ok(())
}
