use axum::{body::Bytes, extract::multipart::Field};
use fs2::FileExt;

pub struct FileUpload {
    pub bytes: Bytes,
    pub filename: String,
}

impl FileUpload {
    pub async fn new(value: Field<'_>) -> Option<Self> {
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
