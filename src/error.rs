// https://www.lpalmieri.com/posts/error-handling-rust/

// IT's advisable to have your code panic when its possible that your code could end up in a bad state. The bad state is not something that's expected to happen occasionally

// Your code after this point needs to rely on not being in this bad state.

// In example, it's understood that a call to a method like unwrap that could panic is meant as a placeholder for the way you'd want your application to handle errors, which can differ based on what the rest of your code is doing.

// Similarly, the unwrap and expect methods are very handy when prototyping berfore you're ready to decide how to handle errors.

// When failure is expected, it's more appropriate to return a Result than to make a panic!.

// WE dont have null values we have option have take Some(T), We use unwrap_or(default)

use core::fmt;

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use deadpool_postgres::PoolError;
use serde::Serialize;
use tokio_postgres::Error;

#[derive(Debug)]
pub enum AppErrorType {
    DbError,
    NotFoundError,
}

#[derive(Debug)]
// Struct type is already defined
pub struct AppError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: AppErrorType,
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub error: String,
}
impl AppError {
    // we are handling the none. function name should match field name
    fn message(&self) -> String {
        match &*self {
            // Error message is found then clone otherwise default message
            AppError {
                cause: _,
                message: Some(message),
                error_type: _,
            } => message.clone(),
            // we are handling the dbError type
            AppError {
                cause: _,
                message: None,
                error_type: AppErrorType::NotFoundError,
            } => "The requested item was not found".to_string(),
            _ => "An unexpected error has occured".to_string(),
        }
    }
    // This db_error is used when we haven't implmented the From trait 

    // pub fn db_error(error: impl ToString) -> AppError {
    //     AppError {
    //         cause: Some(error.to_string()),
    //         message: None,
    //         error_type: AppErrorType::DbError,
    //     }
    // }
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl ResponseError for AppError {
    //error_response and status_code are the provided methods for ResponseError Trait

    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::DbError => (StatusCode::INTERNAL_SERVER_ERROR),
            AppErrorType::NotFoundError => (StatusCode::NOT_FOUND),
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}

// It is a converter used to convert one type to another. Here we are converting the PoolError to AppError
impl From<PoolError> for AppError {
    fn from(error: PoolError) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}
impl From<Error> for AppError {
    fn from(error: Error) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}
