use crate::{response::Result, util::stream_to_file};
use axum::{extract::Multipart, routing::get, Router};
use service::sea_orm::DatabaseConnection;

pub async fn upload_file(mut multipart: Multipart) -> Result<()> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().unwrap().to_string();
        stream_to_file(&name, field).await?;
    }
    Ok(())
}

pub fn route() -> Router<DatabaseConnection> {
    let router = Router::new().route("/upload", get(upload_file));
    router
}
