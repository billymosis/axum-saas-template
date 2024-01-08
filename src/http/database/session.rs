use time::OffsetDateTime;
use uuid::Uuid;

use super::DB;
use crate::http::models::session::{SessionModel, SessionResponse};

use crate::http::{Error, Result};

pub trait Session {
    async fn create_session(
        &self,
        user_id: Uuid,
        data: serde_json::Value,
        expiry_date: OffsetDateTime,
    ) -> Result<SessionResponse>;

    async fn get_session(&self, user_id: &Uuid) -> Result<SessionModel>;
}

impl Session for DB {
    async fn create_session(
        &self,
        user_id: Uuid,
        data: serde_json::Value,
        expiry_date: OffsetDateTime,
    ) -> Result<SessionResponse> {
        let result = sqlx::query_as!(
            SessionResponse,
            r#"
            INSERT INTO sessions (user_id, data, expiry_date)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            user_id,
            data,
            expiry_date,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(result)
    }

    async fn get_session(&self, user_id: &Uuid) -> Result<SessionModel> {
        let result = sqlx::query_as::<_, SessionModel>(
            r#"select id, user_id, data, expiry_date from sessions where id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| Error::Forbidden)?;

        Ok(result)
    }
}
