use askama::Template;
use axum::{
    Router, body::Body, extract::{Multipart, Query, State}, http::{Response, StatusCode, header}, response::{Html, IntoResponse}
};
use log::info;
use tokio_util::io::ReaderStream;

use crate::{
    AppState,
    controllers::{FileType, Filedata},
    error::ApplicationResult,
};

pub async fn get_info(State(state): State<AppState>) -> String {
    state.get_info().await
}

#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTemplate;

pub async fn get_upload_file_page() -> ApplicationResult<impl IntoResponse> {
    let template = UploadTemplate.render()?;

    Ok(Html(template))
}

#[derive(Template)]
#[template(path = "file-display.html")]
struct FileDisplayTemplate<'a> {
    files: &'a [Filedata],
}

pub async fn get_file_display_page(
    State(state): State<AppState>,
) -> ApplicationResult<impl IntoResponse> {
    let files = state.get_all_file_data().await?;
    let template = FileDisplayTemplate { files: &files }.render()?;

    Ok(Html(template))
}

#[derive(serde::Deserialize, Debug)]
pub struct FileDownloadQuery {
    pub filename: String,
}

pub async fn get_download_file(
    State(state): State<AppState>,
    Query(query): Query<FileDownloadQuery>,
) -> ApplicationResult<impl IntoResponse> {
    let file = state.download_file(&query).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    info!("File '{}' found, starting download!", query.filename);
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", query.filename),
            ),
        ],
        body,
    ))
}

pub async fn post_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> ApplicationResult<impl IntoResponse> {
    while let Some(field) = multipart.next_field().await? {
        let file_data = state.upload_file(field).await?;
        info!("File '{}' saved successfully!", &file_data.filename);
    }

    Ok((StatusCode::OK, "File uploaded successfully!"))
}

pub async fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate;

pub async fn get_not_found_page() -> ApplicationResult<impl IntoResponse> {
    let template = NotFoundTemplate;
    Ok((StatusCode::NOT_FOUND, Html(template.render()?)).into_response())
}

static STATIC_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/static");

async fn get_static_handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {

    match STATIC_DIR.get_file(path) {
        Some(file) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .body(Body::from(file.contents()))
                .unwrap()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}


pub fn static_content_router() -> Router<AppState> {
    Router::new().route("/static/{*path}", axum::routing::get(get_static_handler))
}