use axum::extract::multipart::Field;
use fs2::FileExt;
use tokio::{io::AsyncWriteExt, sync::Mutex};

#[derive(Debug)]
pub struct FileUploader {
    #[allow(dead_code)]
    lock_file: std::fs::File,
    folder_path: std::path::PathBuf,
    upload_count: tokio::sync::Mutex<u64>,
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
            upload_count: Mutex::new(0),
        })
    }

    pub fn init(folder_path: &str) -> anyhow::Result<Self> {
        let as_path = std::path::PathBuf::from(folder_path);
        Self::new(as_path)
    }

    pub async fn print_info(&self) {
        println!("{}", self.get_info().await);
    }

    pub async fn get_info(&self) -> String {
        format!(
            "Uploaded {} files to '{}'",
            self.upload_count.lock().await,
            self.folder_path.display()
        )
    }

    pub async fn upload_file(&self, field: Field<'_>) -> anyhow::Result<()> {
        let file_id = {
            let mut count = self.upload_count.lock().await;

            let id = *count;

            *count = count
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Upload counter overflow"))?;

            id
        };
        self.write_file(field, file_id).await
    }

    pub async fn write_file(&self, mut field: Field<'_>, file_id: u64) -> anyhow::Result<()> {
        let default_filename = format!("file_upload_{}", file_id);
        let raw_filename = field.file_name().unwrap_or(&default_filename);
        let safe_name = std::path::Path::new(raw_filename)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(default_filename);

        let mut file_path = self.folder_path.clone();
        file_path.push(safe_name);
        let mut file_handle = tokio::fs::File::create(file_path).await?;

        while let Some(chunk) = field.chunk().await? {
            file_handle.write_all(&chunk).await?;
        }
        file_handle.flush().await?;
        Ok(())
    }
}
