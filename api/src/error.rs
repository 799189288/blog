use axum::response::{IntoResponse, Response};
use migration::DbErr;
use thiserror::Error;
use tokio::io;

use crate::response::CustomResponse;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error("Unauthorized")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
    #[error(transparent)]
    FormRejection(#[from] axum::extract::rejection::FormRejection),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl CustomError {
    pub fn code(&self) -> i32 {
        match self {
            &CustomError::Jwt(_) => 9527,
            &CustomError::Database(_) => 9528,
            _ => -1,
        }
    }
}

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        CustomResponse::<()>::error_with_code(self.code(), &self)
            .to_json()
            .into_response()
    }
}
