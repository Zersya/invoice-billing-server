use argon2::{Config, ThreadMode, Variant, Version};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct OauthAccessToken {
    pub id: Uuid,
    pub access_token: String,
    pub user_id: Uuid,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl OauthAccessToken {
    pub async fn create(
        db: &sqlx::PgPool,
        access_token: String,
        user_id: Uuid,
    ) -> Result<OauthAccessToken, sqlx::Error> {
        // expires at token in 7 days
        let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(7);

        let oauth_access_token = sqlx::query_as!(
            OauthAccessToken,
            r#"
            INSERT INTO oauth_access_tokens (access_token, user_id, expires_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            access_token,
            user_id,
            expires_at
        )
        .fetch_one(db)
        .await?;

        Ok(oauth_access_token)
    }

    pub async fn delete (
        db: &sqlx::PgPool,
        access_token: String,
    ) -> Result<OauthAccessToken, sqlx::Error> {
        let oauth_access_token = sqlx::query_as!(
            OauthAccessToken,
            r#"
            UPDATE oauth_access_tokens
            SET revoked_at = $1
            WHERE access_token = $2
            RETURNING *
            "#,
            chrono::Utc::now().naive_utc(),
            access_token
        )
        .fetch_one(db)
        .await?;

        Ok(oauth_access_token)
    }

    pub async fn get_by_user_id (
        db: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<OauthAccessToken, sqlx::Error> {
        let oauth_access_tokens = sqlx::query_as!(
            OauthAccessToken,
            r#"
            SELECT *
            FROM oauth_access_tokens
            WHERE user_id = $1
            ORDER BY created_at ASC
            "#,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(oauth_access_tokens)
    }

    pub async fn get_count_by_user_id (
        db: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_access_tokens
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(db)
        .await?
        .count;

        match count {
            Some(count) => Ok(count),
            None => Ok(0),
        }
    }
}