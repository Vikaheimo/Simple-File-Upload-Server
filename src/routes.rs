use askama::Template;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use log::{info, warn};

use crate::{AppState, controllers::Filedata};

pub async fn get_info(State(state): State<AppState>) -> String {
    state.get_info().await
}

#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTemplate;

pub async fn get_upload_file_page() -> Result<impl IntoResponse, (StatusCode, String)> {
    let template = UploadTemplate.render().map_err(|e| {
        warn!("Template render error: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {e}"),
        )
    })?;

    Ok(Html(template))
}

#[derive(Template)]
#[template(path = "file-display.html")]
struct FileDisplayTemplate<'a> {
    files: &'a [Filedata],
}

pub async fn get_file_display_page(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let files = state.get_all_file_data().await.map_err(|e| {
        warn!("Directory read error: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Directory read error: {e}"),
        )
    })?;
    let template = FileDisplayTemplate { files: &files }
        .render()
        .map_err(|e| {
            warn!("Template render error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template render error: {e}"),
            )
        })?;

    Ok(Html(template))
}

pub async fn post_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        warn!("Multipart read error: {e}");
        (
            StatusCode::BAD_REQUEST,
            format!("Multipart read error: {e}"),
        )
    })? {
        let file_data = state.upload_file(field).await.map_err(|e| {
            warn!("File save failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("File save failed: {e}"),
            )
        })?;
        info!("File '{}' saved successfully!", &file_data.filename);
    }

    Ok((StatusCode::OK, "File uploaded successfully!"))
}

pub async fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
