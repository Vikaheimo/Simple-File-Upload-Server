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

use axum::{
    Router,
    body::Bytes,
    extract::{Multipart, State, multipart::Field},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use clap::Parser;
use fs2::FileExt;
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub struct FileUpload {
    pub bytes: Bytes,
    pub filename: String,
}

impl FileUpload {
    async fn new(value: Field<'_>) -> Option<Self> {
        let filename = value.file_name()?.to_string();
        let bytes = value.bytes().await.ok()?;

        Some(Self { filename, bytes })
    }
}

#[derive(Debug)]
pub struct FileUploader {
    #[allow(dead_code)]
    lock_file: std::fs::File,
    folder_path: std::path::PathBuf,
    upload_count: u64,
}

impl FileUploader {
    fn new(folder_path: std::path::PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&folder_path)?;

        let mut lockfile_path = folder_path.clone();
        lockfile_path.push(".lock");
        let lock_file = std::fs::File::options()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(lockfile_path)?;
        lock_file.try_lock_exclusive()?;

        Ok(Self {
            lock_file,
            folder_path,
            upload_count: 0,
        })
    }

    pub fn init(folder_path: &str) -> anyhow::Result<Self> {
        let as_path = std::path::PathBuf::from(folder_path);
        Self::new(as_path)
    }

    pub fn print_info(&self) {
        println!("{}", self.get_info());
    }

    pub fn get_info(&self) -> String {
        format!(
            "Uploaded {} files to '{}'",
            self.upload_count,
            self.folder_path.display()
        )
    }

    pub async fn upload_file(&mut self, file: FileUpload) -> anyhow::Result<()> {
        let mut file_path = self.folder_path.clone();
        file_path.push(file.filename);
        tokio::fs::write(file_path, file.bytes).await?;

        self.upload_count = self.upload_count.checked_add(1).ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot upload more files: counter at maximum value of {}",
                self.upload_count
            )
        })?;
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref ENVIRONMENT: Environment = Environment::parse();
}

pub type AppState = Arc<Mutex<FileUploader>>;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(e) => {
            eprintln!(
                "Failed to start server on address {}",
                ENVIRONMENT.server_address
            );

            if ENVIRONMENT.verbose {
                eprintln!("{}", e);
            }
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let shared_state: AppState = Arc::new(Mutex::new(FileUploader::init(&ENVIRONMENT.folder)?));
    let app = Router::new()
        .route("/version", get(version_route))
        .route("/info", get(info_route))
        .route("/upload", post(upload_route))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(&ENVIRONMENT.server_address).await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn info_route(State(state): State<AppState>) -> String {
    state.lock().await.get_info()
}

async fn upload_route(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Multipart read error: {e}"),
        )
    })? {
        let file = FileUpload::new(field).await.ok_or((
            StatusCode::BAD_REQUEST,
            "Invalid file metadata or contents".to_string(),
        ))?;

        let mut uploader = state.lock().await;
        uploader.upload_file(file).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("File save failed: {e}"),
            )
        })?;
    }

    Ok((StatusCode::OK, "File uploaded successfully!"))
}

async fn version_route() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
