use axum::{Router, routing::get};
use clap::Parser;
use fs2::FileExt;

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

#[derive(Debug)]
struct FileUploader {
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
        println!(
            "Uploaded {} files to '{}'",
            self.upload_count,
            self.folder_path.display()
        );
    }
}

lazy_static::lazy_static! {
    static ref ENVIRONMENT: Environment = Environment::parse();
    static ref FILE_UPLOADER: FileUploader = FileUploader::init(&ENVIRONMENT.folder)
        .expect("Failed to initialize FileUploader. This application allows only one running instance per upload folder. Check that the upload folder exists or can be created, that the process has permissions to access it, and that no other instance of this application is currently running and holding the lock file.");
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/version", get(version_route));

    let listener = tokio::net::TcpListener::bind(&ENVIRONMENT.server_address)
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    FILE_UPLOADER.print_info();
    axum::serve(listener, app).await.unwrap();
}

async fn version_route() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
