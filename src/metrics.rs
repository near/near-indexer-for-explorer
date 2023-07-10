use near_o11y::metrics::{
    exponential_buckets, try_create_histogram, try_create_histogram_vec, Histogram, HistogramVec,
};
use once_cell::sync::Lazy;

pub(crate) static HANDLE_MESSAGE_TIME: Lazy<Histogram> = Lazy::new(|| {
    try_create_histogram(
        "near_indexer_for_explorer_handle_message_time",
        "Latency of handling a streamer message",
    )
    .unwrap()
});

pub(crate) static STORE_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    try_create_histogram_vec(
        "near_indexer_for_explorer_store_time",
        "Latency of storing an object in the DB",
        &["object"],
        Some(exponential_buckets(0.001, 1.6, 30).unwrap()),
    )
    .unwrap()
});
