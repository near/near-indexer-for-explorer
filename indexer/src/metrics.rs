use actix_web::{get, App, HttpServer, Responder};
use lazy_static::lazy_static;
use prometheus::{Encoder, IntCounter, IntGauge, Opts};
use tracing::info;

lazy_static! {
    pub(crate) static ref LATEST_BLOCK_HEIGHT: IntGauge = try_create_int_gauge(
        "indexer_explorer_lake_latest_block_height",
        "Height of last processed block"
    )
    .unwrap();
    pub(crate) static ref BLOCK_COUNT: IntCounter = try_create_int_counter(
        "indexer_explorer_lake_block_count",
        "Number of indexed blocks"
    )
    .unwrap();
}

fn try_create_int_gauge(name: &str, help: &str) -> prometheus::Result<IntGauge> {
    let opts = Opts::new(name, help);
    let gauge = IntGauge::with_opts(opts)?;
    prometheus::register(Box::new(gauge.clone()))?;
    Ok(gauge)
}

fn try_create_int_counter(name: &str, help: &str) -> prometheus::Result<IntCounter> {
    let opts = Opts::new(name, help);
    let counter = IntCounter::with_opts(opts)?;
    prometheus::register(Box::new(counter.clone()))?;
    Ok(counter)
}

#[get("/metrics")]
async fn get_metrics() -> impl Responder {
    let mut buffer = Vec::<u8>::new();
    let encoder = prometheus::TextEncoder::new();
    loop {
        match encoder.encode(&prometheus::gather(), &mut buffer) {
            Ok(_) => break,
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    String::from_utf8(buffer.clone()).unwrap()
}

pub(crate) async fn init_server() -> Result<(), std::io::Error> {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3030"))
        .parse()
        .expect("Unable to parse `PORT`");

    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Starting metrics server on http://0.0.0.0:{port}"
    );

    HttpServer::new(|| App::new().service(get_metrics))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
