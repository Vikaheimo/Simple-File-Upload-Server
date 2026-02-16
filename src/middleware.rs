use axum::{body::Body, http::Request, middleware::Next, response::Response};
use log::info;
use std::time::Instant;

pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    let start = Instant::now();
    let response = next.run(req).await;

    let status = response.status();
    let duration = start.elapsed();

    info!(
        "{} {} {} ({} ms)",
        method,
        uri,
        status,
        duration.as_millis()
    );

    response
}
