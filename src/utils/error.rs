use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::{
    error::Error as StdError,
    fmt::{self},
};

#[derive(Debug)]
pub enum Error {
    NotFound,
    Validation(String),
    Conflict(String),
    Unauthorized,
    Forbidden,
    Db(sqlx::Error),
    Unexpected(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "not found"),
            Error::Validation(msg) => write!(f, "validation error: {msg}"),
            Error::Conflict(msg) => write!(f, "conflict: {msg}"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::Forbidden => write!(f, "forbidden"),
            Error::Db(e) => write!(f, "database error: {e}"),
            Error::Unexpected(msg) => write!(f, "unexpected error: {msg}"),
        }
    }
}

impl StdError for Error {}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Error::NotFound,
            other => Error::Db(other),
        }
    }
}

impl From<deadpool::managed::PoolError<deadpool_redis::redis::RedisError>> for Error {
    fn from(err: deadpool::managed::PoolError<deadpool_redis::redis::RedisError>) -> Self {
        Error::Unexpected(format!("redis pool error: {err}"))
    }
}

impl From<deadpool_redis::redis::RedisError> for Error {
    fn from(err: deadpool_redis::redis::RedisError) -> Self {
        Error::Unexpected(format!("redis pool error: {err}"))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Unexpected(format!("serde json error: {err}"))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Unexpected(format!("http error: {err}"))
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::Db(_) | Error::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (code, message) = match self {
            Error::NotFound => ("NOT_FOUND", self.to_string()),
            Error::Validation(_) => ("VALIDATION_ERROR", self.to_string()),
            Error::Conflict(_) => ("CONFLICT", self.to_string()),
            Error::Unauthorized => ("UNAUTHORIZED", self.to_string()),
            Error::Forbidden => ("FORBIDDEN", self.to_string()),
            Error::Db(_) => ("DB_ERROR", self.to_string()),
            Error::Unexpected(_) => ("UNEXPECTED", self.to_string()),
        };

        let body = ErrorBody { code, message };
        HttpResponse::build(self.status_code()).json(body)
    }
}
