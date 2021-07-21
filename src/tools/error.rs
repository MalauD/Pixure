use actix_web::{dev::HttpResponseBuilder, http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceIOError {
    #[error("InsufficientPermissions: cannot {0} resource")]
    InsufficientPermissions(String),
    #[error("DatabaseError: something went wrong with mongodb")]
    DatabaseError(#[from] mongodb::error::Error),
}

impl ResponseError for ResourceIOError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InsufficientPermissions(_) => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).finish()
    }
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("MismatchingCredential: cannot login")]
    MismatchingCredential,
    #[error("DatabaseError: something went wrong with mongodb")]
    DatabaseError(#[from] mongodb::error::Error),
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::MismatchingCredential => StatusCode::UNAUTHORIZED,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).finish()
    }
}
