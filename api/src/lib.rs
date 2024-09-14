mod auth;
mod error;
mod openapi;
mod response;
mod util;
mod v1;
mod validate;

use axum::{http::StatusCode, response::IntoResponse, Router};
use dotenvy::dotenv;
use migration::{extension::postgres::Extension, ConnectionTrait, PostgresQueryBuilder};
use openapi::ApiDoc;
use service::sea_orm::{ConnectOptions, Database, DatabaseBackend, Statement};
use std::{env, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await.unwrap();
    // 先创建uuid扩展
    let stmt = Extension::create()
        .name(r#""uuid-ossp""#)
        .if_not_exists()
        .to_string(PostgresQueryBuilder);
    db.execute(Statement::from_string(DatabaseBackend::Postgres, stmt))
        .await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let app = Router::new()
        .merge(Scalar::with_url("/", ApiDoc::openapi()))
        .nest("/api/v1", v1::article::route())
        .merge(v1::user::route())
        .merge(v1::upload::route())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .with_state(db)
        .fallback(handle_rejection);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_rejection() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}
