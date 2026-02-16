// Arithmetic
#![deny(arithmetic_overflow)] // Prevent operations that would cause integer overflow
#![deny(clippy::checked_conversions)] // Suggest using checked conversions between numeric types
#![deny(clippy::cast_possible_truncation)] // Detect when casting might truncate a value
#![deny(clippy::cast_sign_loss)] // Detect when casting might lose sign information
#![deny(clippy::cast_possible_wrap)] // Detect when casting might cause value to wrap around
#![deny(clippy::cast_precision_loss)] // Detect when casting might lose precision
#![deny(clippy::integer_division)] // Highlight potential bugs from integer division truncation
#![deny(clippy::arithmetic_side_effects)] // Detect arithmetic operations with potential side effects
#![deny(clippy::unchecked_time_subtraction)] // Ensure duration subtraction won't cause underflow

// Unwraps
#![warn(clippy::unwrap_used)] // Discourage using .unwrap() which can cause panics
#![warn(clippy::expect_used)] // Discourage using .expect() which can cause panics
#![deny(clippy::panicking_unwrap)] // Prevent unwrap on values known to cause panics
#![deny(clippy::option_env_unwrap)] // Prevent unwrapping environment variables which might be absent

// Array indexing
#![deny(clippy::indexing_slicing)] // Avoid direct array indexing and use safer methods like .get()

// Path handling
#![deny(clippy::join_absolute_paths)] // Prevent issues when joining paths with absolute paths

// Serialization issues
#![deny(clippy::serde_api_misuse)] // Prevent incorrect usage of Serde's serialization/deserialization API

// Unbounded input
#![deny(clippy::uninit_vec)] // Prevent creating uninitialized vectors which is unsafe

// Unsafe code detection
#![deny(unnecessary_transmutes)] // Prevent unsafe transmutation
#![deny(clippy::transmute_ptr_to_ref)] // Prevent unsafe transmutation from pointers to references
#![deny(clippy::transmute_undefined_repr)] // Detect transmutes with potentially undefined representations

pub mod controllers;
pub mod error;
pub mod routes;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use clap::Parser;
use log::{error, info};
use std::sync::Arc;

use crate::{controllers::FileUploader, routes::get_not_found_page};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Environment {
    /// The address where the server should bind to
    #[arg(short, long, default_value_t=String::from("localhost:3000"))]
    pub server_address: String,

    /// Folder where uploads are stored at
    #[arg(short, long, default_value_t=String::from("./uploads"))]
    pub folder: String,

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

lazy_static::lazy_static! {
    static ref ENVIRONMENT: Environment = Environment::parse();
}

pub type AppState = Arc<FileUploader>;

#[tokio::main]
async fn main() {
    colog::init();
    match run().await {
        Ok(_) => (),
        Err(e) => {
            error!(
                "Failed to start server on address {}",
                ENVIRONMENT.server_address
            );

            if ENVIRONMENT.verbose {
                error!("{}", e);
            }
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let shared_state: AppState = Arc::new(FileUploader::init(&ENVIRONMENT.folder)?);
    let app = Router::new()
        .route("/version", get(routes::get_version))
        .route("/info", get(routes::get_info))
        .route("/upload", post(routes::post_upload))
        .route("/upload", get(routes::get_upload_file_page))
        .route("/download", get(routes::get_download_file))
        .route("/", get(routes::get_file_display_page))
        .fallback(get_not_found_page)
        .layer(DefaultBodyLimit::disable())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(&ENVIRONMENT.server_address).await?;
    info!("Server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
