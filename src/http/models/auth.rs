use crate::http::utils::password::validate_password;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use time::OffsetDateTime;

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct EmailTokenX {
    pub id: String,
    pub active_expires: chrono::NaiveDateTime,
    pub user_id: String,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct EmailToken {
    pub id: String,
    pub active_expires: chrono::NaiveDateTime,
    pub user_id: uuid::Uuid,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct PasswordToken {
    pub id: String,
    pub active_expires: chrono::NaiveDateTime,
    pub user_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct ResetPayload {
    #[validate(email)]
    pub email: String,
}

lazy_static! {
    static ref RE_SPECIAL_CHAR: Regex = Regex::new("^.*?[@$!%*?&].*$").unwrap();
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct VerifyResetPasswordPayload {
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
