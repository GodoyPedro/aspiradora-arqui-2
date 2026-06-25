use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::api::ErrorResponse;
use crate::domain::{DomainError, ErrorKind};

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Domain(DomainError),
}

impl ApiError {
    pub fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::BadRequest(rejection.body_text())
    }
}

impl From<DomainError> for ApiError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "BAD_REQUEST".to_string(),
                    message,
                    current_state: None,
                }),
            )
                .into_response(),
            Self::Domain(error) => {
                let status = match error.kind {
                    ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
                    ErrorKind::Conflict => StatusCode::CONFLICT,
                };

                (
                    status,
                    Json(ErrorResponse {
                        code: error.code.to_string(),
                        message: error.message,
                        current_state: error.current_state.map(|state| state.to_string()),
                    }),
                )
                    .into_response()
            }
        }
    }
}
