use crate::http::Error;
use axum::{
    async_trait,
    extract::{
        rejection::{FormRejection, JsonRejection},
        FromRequest, Request,
    },
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Form, Json, RequestExt,
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonOrForm<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for JsonOrForm<T>
where
    Json<T>: FromRequest<()>,
    Form<T>: FromRequest<()>,
    T: 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| StatusCode::BAD_REQUEST.into_response())?;

        if content_type.starts_with("application/json") {
            let Json(payload) = req
                .extract::<Json<T>, _>()
                .await
                .map_err(|err| err.into_response())?;

            Ok(Self(payload))
        } else if content_type.starts_with("application/x-www-form-urlencoded") {
            let Form(payload) = req
                .extract::<Form<T>, _>()
                .await
                .map_err(|err| err.into_response())?;

            Ok(Self(payload))
        } else {
            Err(StatusCode::BAD_REQUEST.into_response())
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedBody<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedBody<T>
where
    T: DeserializeOwned + Validate + std::fmt::Debug,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());
        match content_type {
            Some(str) => {
                if str.starts_with("application/json") {
                    let Json(value) = Json::<T>::from_request(req, state).await?;
                    let x = value.validate();
                    match x {
                        Ok(_) => Ok(ValidatedBody(value)),
                        Err(e) => Err(Error::ValidationErrors(e)),
                    }
                } else if str.starts_with("application/x-www-form-urlencoded") {
                    let Form(value) = Form::<T>::from_request(req, state).await?;
                    value.validate()?;
                    Ok(ValidatedBody(value))
                } else {
                    Err(Error::BadRequest)
                }
            }
            None => Err(Error::BadRequest),
        }
    }
}
