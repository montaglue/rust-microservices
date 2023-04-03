use axum::{
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;

pub type ServiceResponse<T> = Result<Json<T>, ServiceError>;

pub struct ServiceError(anyhow::Error);

impl<E> From<E> for ServiceError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error message: {}", self.0),
        )
            .into_response()
    }
}
