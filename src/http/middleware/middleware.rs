use std::str::FromStr;

use axum::{
    extract::{Request, State},
    middleware::Next,
};
use time::OffsetDateTime;
use tower_cookies::Cookies;
use uuid::Uuid;

use super::super::{Error, Result};
use crate::http::{database::session::Session, AppState};

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: Uuid,
}

pub async fn auth_middleware(
    cookies: Cookies,
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<axum::response::Response> {
    let session_id = cookies
        .get("session_id")
        .ok_or_else(|| Error::NotFound)?
        .value_trimmed()
        .to_owned();

    let uuid = uuid::Uuid::from_str(&session_id)?;
    let session = state.db.get_session(&uuid).await?;

    if session.expiry_date > OffsetDateTime::now_utc() {
        request.extensions_mut().insert(AuthContext {
            user_id: session.user_id,
        });
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(Error::Forbidden)
    }
}
