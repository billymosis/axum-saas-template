use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::error::DatabaseError;
use validator::{ValidationError, ValidationErrorsKind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("authentication required")]
    Unauthorized,

    #[error("email must be verified")]
    NotVerified,

    #[error("user may not perform that action")]
    Forbidden,

    #[error("request path not found")]
    NotFound,

    #[error("bad request")]
    BadRequest,

    #[error("error in the request body")]
    UnprocessableEntity { errors: Vec<FieldError> },

    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    #[error("an error occurred with the uuid")]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    ValidationErrors(#[from] validator::ValidationErrors),

    #[error(transparent)]
    ValidationError(#[from] validator::ValidationError),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    message: String,
    domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FieldErrors(Vec<FieldError>);

impl FieldError {
    pub fn new(domain: Option<&str>, message: &str) -> Self {
        match domain {
            Some(val) => FieldError {
                domain: Some(val.to_owned()),
                message: message.to_owned(),
            },
            None => FieldError {
                domain: None,
                message: message.to_owned(),
            },
        }
    }
}

impl Error {
    pub fn unprocessable_entity(error: FieldError) -> Self {
        Self::UnprocessableEntity {
            errors: vec![error],
        }
    }

    // pub fn unprocessable_entities(errors: Vec<FieldError>) -> Self {
    //     Self::UnprocessableEntity { errors }
    // }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized | Self::NotVerified { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ReqwestError(_) | Self::Uuid(_) | Self::Sqlx(_) | Self::Anyhow(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::BadRequest
            | Self::ValidationErrors { .. }
            | Self::ValidationError { .. }
            | Self::AxumFormRejection { .. }
            | Self::JsonRejection { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<&ValidationError> for FieldError {
    fn from(value: &ValidationError) -> Self {
        Self {
            message: value
                .message
                .clone()
                .map_or_else(|| "Unknown Error".to_owned(), |x| x.into_owned()),
            domain: Some(value.code.to_string()),
        }
    }
}

#[derive(serde::Serialize)]
struct ClientError {
    message: String,
    errors: Option<Vec<FieldError>>,
}

#[derive(serde::Serialize)]
struct ClientErrorResponse {
    error: ClientError,
}

trait ErrorResponse<T> {
    fn new(errors: T, message: &str) -> Self;
}

impl ClientErrorResponse {
    fn new(errors: FieldError, message: &str) -> Self {
        Self {
            error: ClientError {
                errors: Some(vec![errors]),
                message: message.to_owned(),
            },
        }
    }

    fn new_message(message: &str) -> Self {
        Self {
            error: ClientError {
                errors: None,
                message: message.to_owned(),
            },
        }
    }

    fn new_from_vec(errors: Vec<FieldError>, message: &str) -> Self {
        Self {
            error: ClientError {
                errors: Some(errors),
                message: message.to_owned(),
            },
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{:?}", self);
        match self {
            Self::NotVerified => {
                return (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        FieldError {
                            message: "Email not verified".to_owned(),
                            domain: Some("email".to_owned()),
                        },
                        "Not Verified",
                    )),
                )
                    .into_response();
            }

            Self::UnprocessableEntity { errors } => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(ClientErrorResponse::new_from_vec(
                        errors,
                        "Unprocessable entity",
                    )),
                )
                    .into_response();
            }

            Self::Unauthorized => {
                return (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        FieldError::new(Some("auth"), "Invalid username or password"),
                        "Invalid user credential",
                    )),
                )
                    .into_response();
            }

            Self::Forbidden => {
                return (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        FieldError::new(Some("auth"), "Not Permitted"),
                        "Unauthorized",
                    )),
                )
                    .into_response();
            }

            Self::Sqlx(ref e) => {
                error!("Database Error: {:?}", e);
                (
                    self.status_code(),
                    Json(ClientErrorResponse::new_message("Internal server error")),
                )
                    .into_response()
            }

            Self::Anyhow(ref e) => {
                error!("Anyhow Error: {:?}", e);
                (
                    self.status_code(),
                    Json(ClientErrorResponse::new_message("Internal server error")),
                )
                    .into_response()
            }

            Self::ValidationErrors(ref e) => {
                let mut messages: Vec<ValidationError> = vec![];
                for value in e.errors().values() {
                    match value {
                        ValidationErrorsKind::Struct(v) => {
                            let nested_error = v
                                .field_errors()
                                .into_values()
                                .into_iter()
                                .map(|q| q.clone())
                                .flatten()
                                .collect::<Vec<ValidationError>>();
                            messages.clone_from(&nested_error)
                        }
                        ValidationErrorsKind::List(v) => {
                            for (_, z) in v {
                                let nested_error = z
                                    .field_errors()
                                    .into_values()
                                    .into_iter()
                                    .map(|q| q.clone())
                                    .flatten()
                                    .collect::<Vec<ValidationError>>()
                                    .iter()
                                    .map(|y| y.clone())
                                    .collect::<Vec<_>>();
                                messages.clone_from(&nested_error)
                            }
                        }
                        ValidationErrorsKind::Field(v) => messages.clone_from(v),
                    }
                }
                (
                    self.status_code(),
                    Json(ClientErrorResponse::new_from_vec(
                        messages
                            .iter()
                            .map(FieldError::from)
                            .collect::<Vec<FieldError>>(),
                        "Form error",
                    )),
                )
                    .into_response()
            }

            Self::ValidationError(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(e.into(), "Form error")),
            )
                .into_response(),

            Self::AxumFormRejection(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(
                    FieldError::new(None, &e.to_string()),
                    "Form type error",
                )),
            )
                .into_response(),

            Self::JsonRejection(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(
                    FieldError::new(Some("json"), &e.body_text()),
                    "Form error",
                )),
            )
                .into_response(),

            Self::BadRequest => (
                self.status_code(),
                Json(ClientErrorResponse::new_message("Bad request")),
            )
                .into_response(),

            Self::ReqwestError(ref e) => {
                error!("Reqwest Error: {:?}", e);
                (
                    self.status_code(),
                    Json(ClientErrorResponse::new_message("Internal server error")),
                )
                    .into_response()
            }

            Self::NotFound => (
                self.status_code(),
                Json(ClientErrorResponse::new_message("Not found")),
            )
                .into_response(),

            _ => (
                (StatusCode::INTERNAL_SERVER_ERROR),
                Json(ClientErrorResponse::new_message("Internal server error")),
            )
                .into_response(),
        }
    }
}

/// A little helper trait for more easily converting database constraint errors into API errors.
///
/// ```rust,ignore
/// let user_id = sqlx::query_scalar!(
///     r#"insert into "user" (username, email, password_hash) values ($1, $2, $3) returning user_id"#,
///     username,
///     email,
///     password_hash
/// )
///     .fetch_one(&ctxt.db)
///     .await
///     .on_constraint("user_username_key", |_| Error::unprocessable_entity([("username", "already taken")]))?;
pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
