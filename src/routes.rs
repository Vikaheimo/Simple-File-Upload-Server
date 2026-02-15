use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{AppState, controllers::FileUpload};

pub async fn get_info(State(state): State<AppState>) -> String {
    state.lock().await.get_info()
}

pub async fn post_upload(
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

pub async fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
