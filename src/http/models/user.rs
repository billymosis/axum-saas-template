use crate::http::utils::password::validate_password;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct LoginPayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub password: String,
}

lazy_static! {
    static ref RE_SPECIAL_CHAR: Regex = Regex::new("^.*?[@$!%*?&].*$").unwrap();
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserRequest {
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(
        custom(
            function = "validate_password",
            message = "Must Contain At Least One Upper Case, Lower Case and Number. Dont use spaces."
        ),
        regex(
            path = "RE_SPECIAL_CHAR",
            message = "Must Contain At Least One Special Character"
        )
    )]
    pub password: String,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub password_hash: String,
    pub created_at: sqlx::types::time::OffsetDateTime,
    pub updated_at: sqlx::types::time::OffsetDateTime,
}

impl From<UserModel> for UserResponse {
    fn from(user_model: UserModel) -> Self {
        UserResponse {
            id: user_model.id,
            username: user_model.username,
            email: user_model.email,
            email_verified: user_model.email_verified,
            created_at: user_model.created_at,
            updated_at: user_model.updated_at,
        }
    }
}
