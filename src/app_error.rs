//file does not exist
//word does not exist
use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum AppError {
    #[display(fmt = "Source file has invalid content.")]
    InvalidFileContent,
    #[display(fmt = "Word does not exist.")]
    WordDoesNotExist,
    #[display(fmt = "Unexpected internal error.")]
    UnexpectedError,
}

impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::InvalidFileContent | AppError::UnexpectedError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::WordDoesNotExist => StatusCode::BAD_REQUEST,
        }
    }
}
