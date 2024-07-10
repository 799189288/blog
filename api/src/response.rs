use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::CustomError;
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CustomResponse<T: Serialize> {
    pub data: Option<T>,
    pub message: String,
    pub code: i32,
}

pub type JsonResponse<T> = Json<CustomResponse<T>>;

impl<T: Serialize> CustomResponse<T> {
    pub fn new(data: Option<T>, message: &str, code: i32) -> Self {
        let message = message.to_string();
        Self {
            data,
            message,
            code,
        }
    }

    pub fn ok(data: T) -> Self {
        Self::new(Some(data), "OK", 0)
    }

    pub fn failed(code: i32, msg: &str) -> Self {
        Self::new(None, msg, code)
    }
    pub fn error_with_code(code: i32, e: &CustomError) -> Self {
        Self::failed(code, &e.to_string())
    }
    pub fn to_json(self) -> JsonResponse<T> {
        Json(self)
    }
}


