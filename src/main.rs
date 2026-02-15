use axum::{Router, routing::get};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Environment {
    /// The address where the server should bind to
    #[arg(short, long, default_value_t=String::from("localhost:3000"))]
    pub server_address: String,

    /// Folder where uploads are stored at
    #[arg(short, long, default_value_t=String::from("./uploads"))]
    pub folder: String,
}

lazy_static::lazy_static! {
    static ref ENVIRONMENT: Environment = Environment::parse();
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/version", get(version_route));

    let listener = tokio::net::TcpListener::bind(&ENVIRONMENT.server_address)
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn version_route() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
