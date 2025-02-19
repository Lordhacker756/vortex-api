use std::{convert::Infallible, time::Duration};

use axum::{
    extract::Path,
    response::{sse::Event, Sse},
};
use axum_extra::{headers, TypedHeader};

use tokio::stream;
use tokio_stream::{Stream, StreamExt};

#[axum::debug_handler]
pub async fn start_sse(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // Create a stream that emits a new event every second
    let stream =
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(|_| Ok(Event::default().data("ping").id("id").event("message")));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

pub async fn get_poll_by_id(Path(poll_id): Path<String>) -> String {
    poll_id
}

pub async fn create_new_poll() {}

pub async fn get_all_polls() {}

pub async fn close_poll_by_id(Path(poll_id): Path<String>) -> String {
    poll_id
}

pub async fn reset_poll_by_id(Path(poll_id): Path<String>) -> String {
    poll_id
}
