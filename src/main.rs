use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/version", get(version_route));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn version_route() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
