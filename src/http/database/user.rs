use time::OffsetDateTime;

use super::DB;
use crate::http::error::{FieldError, ResultExt};
use crate::http::models::auth::{EmailToken, PasswordToken};
use crate::http::models::user::UserModel;

use crate::http::{Error, Result};

pub trait User {
    async fn find_user_by_email(&self, email: &str) -> Result<UserModel>;
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<uuid::Uuid>;
    async fn insert_verification_token(
        &self,
        token: &str,
        expires: OffsetDateTime,
        user_id: &uuid::Uuid,
    ) -> Result<()>;
    async fn insert_reset_password_token(
        &self,
        token: &str,
        expires: OffsetDateTime,
        user_id: &uuid::Uuid,
    ) -> Result<()>;
    async fn verify_user(&self, token: &uuid::Uuid) -> Result<()>;
    async fn reset_user_password(&self, user_id: &uuid::Uuid, password: &str) -> Result<()>;
    async fn delete_email_token(&self, token: &str) -> Result<()>;
    async fn delete_reset_password_token(&self, token: &str) -> Result<()>;
    async fn get_user_from_email_token(&self, token: &str) -> Result<EmailToken>;
    async fn get_user_from_reset_password_token(&self, token: &str) -> Result<PasswordToken>;
}

impl User for DB {
    async fn find_user_by_email(&self, email: &str) -> Result<UserModel> {
        let user = sqlx::query_as::<_, UserModel>(
            r#"
                SELECT *
                FROM users
                WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| {
            Error::unprocessable_entity(FieldError::new(Some("email"), "email does not exist"))
        })?;

        Ok(user)
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<uuid::Uuid> {
        let user = sqlx::query!(
            r#"
        insert into users (username, email, password_hash) values ($1, $2, $3) returning id 
        "#,
            username,
            email,
            password_hash
        )
        .fetch_one(&self.db)
        .await
        .on_constraint("users_username_key", |_| {
            Error::unprocessable_entity(FieldError::new(Some("username"), "username taken"))
        })
        .on_constraint("users_email_key", |_| {
            Error::unprocessable_entity(FieldError::new(Some("email"), "email taken"))
        })?;
        Ok(user.id)
    }

    async fn insert_verification_token(
        &self,
        token: &str,
        expires: OffsetDateTime,
        user_id: &uuid::Uuid,
    ) -> Result<()> {
        sqlx::query!(
            r#"insert into email_verification_token (id, active_expires, user_id) values ($1, $2, $3)"#,
            token,
            expires,
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn insert_reset_password_token(
        &self,
        token: &str,
        expires: OffsetDateTime,
        user_id: &uuid::Uuid,
    ) -> Result<()> {
        sqlx::query!(
            r#"insert into password_reset_token (id, active_expires, user_id) values ($1, $2, $3)"#,
            token,
            expires,
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn verify_user(&self, user_id: &uuid::Uuid) -> Result<()> {
        sqlx::query!(
            r#"update users set email_verified = true where id = ($1)"#,
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn delete_email_token(&self, token: &str) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM email_verification_token WHERE id = ($1)"#,
            token,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_user_from_email_token(&self, token: &str) -> Result<EmailToken> {
        let row = sqlx::query_as!(
            EmailToken,
            r#"select * from email_verification_token where id = ($1)"#,
            token,
        )
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    async fn get_user_from_reset_password_token(&self, token: &str) -> Result<PasswordToken> {
        let row = sqlx::query_as!(
            PasswordToken,
            r#"select * from password_reset_token where id = ($1)"#,
            token,
        )
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    async fn delete_reset_password_token(&self, token: &str) -> Result<()> {
        sqlx::query!(r#"DELETE FROM password_reset_token WHERE id = ($1)"#, token,)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn reset_user_password(&self, user_id: &uuid::Uuid, password: &str) -> Result<()> {
        sqlx::query!(
            r#"update users set password_hash = ($2) where id = ($1)"#,
            user_id,
            password,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
