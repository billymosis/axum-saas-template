use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use log::error;
use serde_json::json;
use sqlx::error::DatabaseError;
use std::borrow::Cow;
use std::collections::HashMap;

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
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

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

impl Error {
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }
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

#[derive(serde::Serialize)]
struct ClientError<T> {
    message: String,
    errors: T,
}

#[derive(serde::Serialize)]
struct ClientErrorResponse<T> {
    error: ClientError<T>,
}

impl<T> ClientErrorResponse<T> {
    fn new(errors: T, message: &str) -> Self {
        Self {
            error: ClientError {
                errors,
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
                        json!({"email": "Email not verified"}),
                        "Not Verified",
                    )),
                )
                    .into_response();
            }

            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(ClientErrorResponse::new(errors, "Unprocessable entity")),
                )
                    .into_response();
            }

            Self::Unauthorized => {
                return (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        json!({"auth": "Invalid username or password"}),
                        "Invalid user credential",
                    )),
                )
                    .into_response();
            }

            Self::Forbidden => {
                return (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        json!({"auth": "Not permitted"}),
                        "Unauthorized",
                    )),
                )
                    .into_response();
            }

            Self::Sqlx(ref e) => {
                error!("Database Error: {:?}", e);
                (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        e.to_string(),
                        "Internal server error",
                    )),
                )
                    .into_response()
            }

            Self::Anyhow(ref e) => {
                error!("Anyhow Error: {:?}", e);
                (
                    self.status_code(),
                    Json(ClientErrorResponse::new(
                        json!({"general": self.to_string()}),
                        "Error",
                    )),
                )
                    .into_response()
            }

            Self::ValidationErrors(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(e, "Form error")),
            )
                .into_response(),

            Self::ValidationError(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(e, "Form error")),
            )
                .into_response(),

            Self::AxumFormRejection(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(
                    json!({"type": e.to_string()}),
                    "Form type error",
                )),
            )
                .into_response(),

            Self::JsonRejection(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(e.body_text(), "Form error")),
            )
                .into_response(),

            Self::BadRequest => (
                self.status_code(),
                Json(ClientErrorResponse::new(
                    json!({"payload": "Bad request"}),
                    "Bad request",
                )),
            )
                .into_response(),

            Self::ReqwestError(ref e) => (
                self.status_code(),
                Json(ClientErrorResponse::new(e.to_string(), "Request error")),
            )
                .into_response(),


            Self::NotFound => (
                self.status_code(),
                Json(ClientErrorResponse::new(
                    json!({"url": "Not found"}),
                    "Not found",
                )),
            )
                .into_response(),


            _ => (
                (StatusCode::INTERNAL_SERVER_ERROR),
                Json(ClientErrorResponse::new(json!({}), "Internal server error")),
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
