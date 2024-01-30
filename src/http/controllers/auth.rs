use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use log::error;
use serde_json::json;
use time::OffsetDateTime;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use crate::http::{
    database::{session::Session, user::User},
    error::{Error, FieldError},
    models::{
        auth::{ResetPayload, VerifyResetPasswordPayload},
        user::{LoginPayload, UserRequest, UserResponse},
    },
    services::email::{send_reset_password_email, send_verification_email},
    utils::{
        extractor::ValidatedBody, password::{hash_password, verify_password}, response_wrapper::JsonData, token_generator::generate_verification_token
    },
    AppState,
};
pub type Result<T, E = Error> = std::result::Result<T, E>;

async fn register_handler(
    State(state): State<AppState>,
    ValidatedBody(payload): ValidatedBody<UserRequest>,
) -> Result<impl IntoResponse> {
    let password_hash = hash_password(payload.password).await?;

    let user_id = uuid::Uuid::new_v4();
    let id = state
        .db
        .create_user(&user_id, &payload.username, &payload.email, &password_hash)
        .await?;

    let expires_time = OffsetDateTime::now_utc().saturating_add(time::Duration::seconds(
        state.config.email_token_time.clone() as i64,
    ));

    let token = generate_verification_token(8);

    state
        .db
        .insert_verification_token(&token, expires_time, &id)
        .await?;

    send_verification_email(
        &payload.username,
        &payload.email,
        state.reqwest,
        &token,
        state.config,
    )
    .await?;

    Ok((StatusCode::CREATED).into_response())
}

async fn send_reset_token(
    State(state): State<AppState>,
    ValidatedBody(payload): ValidatedBody<ResetPayload>,
) -> Result<impl IntoResponse> {
    let user = state.db.find_user_by_email(&payload.email).await?;
    if !user.email_verified {
        return Err(Error::NotVerified);
    }

    let expires_time = OffsetDateTime::now_utc().saturating_add(time::Duration::seconds(
        state.config.email_token_time.clone() as i64,
    ));

    let token = generate_verification_token(8);

    state
        .db
        .insert_reset_password_token(&token, expires_time, &user.id)
        .await?;

    send_reset_password_email(
        &user.username,
        &payload.email,
        state.reqwest,
        &token,
        state.config,
    )
    .await?;

    Ok((StatusCode::NO_CONTENT).into_response())
}

async fn verify_email_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse> {
    let user = state
        .db
        .get_user_from_email_token(&token)
        .await
        .map_err(|e| {
            error!("error: {:?}", e);
            Error::unprocessable_entity(FieldError::new(None, "token not found"))
        })?;

    if user.active_expires < Utc::now().naive_utc() {
        Err(Error::unprocessable_entity(FieldError::new(
            None,
            "token expired",
        )))?
    }

    state.db.verify_user(&user.user_id).await?;

    state.db.delete_email_token(&token).await?;

    Ok((StatusCode::NO_CONTENT).into_response())
}

async fn verify_reset_password_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
    ValidatedBody(payload): ValidatedBody<VerifyResetPasswordPayload>,
) -> Result<impl IntoResponse> {
    let user = state
        .db
        .get_user_from_reset_password_token(&token)
        .await
        .map_err(|e| {
            error!("error: {:?}", e);
            Error::unprocessable_entity(FieldError::new(None, "token not found"))
        })?;

    if user.active_expires < Utc::now().naive_utc() {
        Err(Error::unprocessable_entity(FieldError::new(
            None,
            "token expired",
        )))?
    }

    let password_hash = hash_password(payload.password).await?;

    state
        .db
        .reset_user_password(&user.user_id, &password_hash)
        .await?;

    state.db.delete_reset_password_token(&token).await?;

    Ok((StatusCode::NO_CONTENT).into_response())
}

async fn login_handler(
    cookies: Cookies,
    State(state): State<AppState>,
    ValidatedBody(payload): ValidatedBody<LoginPayload>,
) -> Result<impl IntoResponse> {
    let expires_time = OffsetDateTime::now_utc().saturating_add(time::Duration::seconds(
        state.config.short_session_time.clone() as i64,
    ));

    let user = state.db.find_user_by_email(&payload.email).await?;
    if !user.email_verified {
        return Err(Error::NotVerified);
    }

    verify_password(payload.password, user.password_hash.to_owned()).await?;

    let result = state
        .db
        .create_session(
            uuid::Uuid::new_v4(),
            user.id,
            json!({"settings": "DUMMY"}),
            expires_time,
        )
        .await?;

    let session = Cookie::build(("session_id", result.id.to_string()))
        .path("/")
        .http_only(true)
        .expires(expires_time)
        .same_site(SameSite::Lax);
    cookies.add(session.into());
    Ok(((StatusCode::OK), JsonData(UserResponse::from(user), None)).into_response())
}

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/verify-email/:token", get(verify_email_token))
        .route("/reset-password", post(send_reset_token))
        .route("/reset-password/:token", post(verify_reset_password_token))
        .route("/register", post(register_handler))
        .with_state(state)
}
