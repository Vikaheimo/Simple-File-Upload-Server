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

static FILE_UPLOADER_CREATE_ERROR_MESSAGE: &str = "Failed to initialize FileUploader. Ensure the upload folder exists or can be created, the process has permission to access it, and no other instance is running and holding the lock file.";

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

pub struct FileUpload {
    pub bytes: Bytes,
    pub filename: String,
}

impl FileUpload {
    async fn new(value: Field<'_>) -> Option<Self> {
        let filename = value.file_name()?.to_string();
        let bytes = value.bytes().await.unwrap();

        Some(Self { filename, bytes })
    }
}

#[derive(Debug)]
pub struct FileUploader {
    #[allow(dead_code)]
    lock_file: std::fs::File,
    folder_path: std::path::PathBuf,
    upload_count: u32,
}

impl FileUploader {
    fn new(folder_path: std::path::PathBuf) -> Option<Self> {
        std::fs::create_dir_all(&folder_path).ok()?;

        let mut lockfile_path = folder_path.clone();
        lockfile_path.push(".lock");
        let lock_file = std::fs::File::options()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(lockfile_path)
            .ok()?;
        lock_file.try_lock_exclusive().ok()?;

        Some(Self {
            lock_file,
            folder_path,
            upload_count: 0,
        })
    }

    pub fn init(folder_path: &str) -> Option<Self> {
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

    pub async fn upload_file(&mut self, file: FileUpload) -> Option<()> {
        let mut file_path = self.folder_path.clone();
        file_path.push(file.filename);
        tokio::fs::write(file_path, file.bytes).await.ok()?;

        self.upload_count += 1;
        Some(())
    }
}

lazy_static::lazy_static! {
    static ref ENVIRONMENT: Environment = Environment::parse();
}

pub type AppState = Arc<Mutex<FileUploader>>;

#[tokio::main]
async fn main() {
    let shared_state: AppState = Arc::new(Mutex::new(
        FileUploader::init(&ENVIRONMENT.folder).expect(FILE_UPLOADER_CREATE_ERROR_MESSAGE),
    ));
    let app = Router::new()
        .route("/version", get(version_route))
        .route("/info", get(info_route))
        .route("/upload", post(upload_route))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(&ENVIRONMENT.server_address)
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn info_route(State(state): State<AppState>) -> String {
    state.lock().await.get_info()
}

async fn upload_route(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    loop {
        let mut uploader = state.lock().await;
        let next_field = multipart.next_field().await;
        match next_field {
            Ok(None) => return (StatusCode::OK, "File uploaded successfully!").into_response(),
            Ok(Some(field)) => {
                let file = FileUpload::new(field).await.unwrap();
                uploader.upload_file(file).await.unwrap();
            }
            Err(_) => {
                return (StatusCode::NOT_ACCEPTABLE, "File uploading failed!").into_response();
            }
        };
    }
}

async fn version_route() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
