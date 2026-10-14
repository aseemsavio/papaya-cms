use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Not found")]
    NotFound,

    #[error("Invalid frontmatter")]
    InvalidFrontmatter,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("S3 error: {0}")]
    S3(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AWS SDK error: {0}")]
    AwsSdk(#[from] aws_sdk_s3::Error),

    #[error("AWS config error: {0}")]
    AwsConfig(String),

    #[error("Template engine error: {0}")]
    Tera(#[from] tera::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::InvalidFrontmatter => (StatusCode::BAD_REQUEST, "Invalid frontmatter"),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::S3(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::Template(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::Cache(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error"),
            AppError::Yaml(_) => (StatusCode::BAD_REQUEST, "YAML parsing error"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            AppError::AwsSdk(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AWS SDK error"),
            AppError::AwsConfig(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AWS config error"),
            AppError::Tera(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Template engine error"),
            AppError::Git(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Git error"),
            AppError::Anyhow(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;