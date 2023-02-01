use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub code: String,
    pub status: String,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Verification {
    pub async fn create(
        db: &sqlx::PgPool,
        user_id: Option<Uuid>,
        customer_id: Option<Uuid>,
        code: &String,
    ) -> Result<Verification, sqlx::Error> {
        let verification = sqlx::query_as!(
            Verification,
            r#"
            INSERT INTO verifications (user_id, customer_id, code, status)
            VALUES ($1, $2, $3, 'pending')
            RETURNING *
            "#,
            user_id,
            customer_id,
            code
        )
        .fetch_one(db)
        .await?;

        Ok(verification)
    }

    pub async fn get_by_id(db: &sqlx::PgPool, id: Uuid) -> Result<Verification, sqlx::Error> {
        let verification = sqlx::query_as!(
            Verification,
            r#"
            SELECT *
            FROM verifications
            WHERE id = $1
            AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(verification)
    }

    pub async fn get_by_user_id(
        db: &sqlx::PgPool,
        user_id: &Uuid,
    ) -> Result<Verification, sqlx::Error> {
        let verification = sqlx::query_as!(
            Verification,
            r#"
            SELECT *
            FROM verifications
            WHERE user_id = $1
            AND status = 'pending'
            AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(verification)
    }

    pub async fn update_status(
        db: &sqlx::PgPool,
        id: &Uuid,
        status: &String,
    ) -> Result<Verification, sqlx::Error> {
        let verification = sqlx::query_as!(
            Verification,
            r#"
            UPDATE verifications
            SET status = $2
            WHERE id = $1
            RETURNING *
            "#,
            id,
            status
        )
        .fetch_one(db)
        .await?;

        Ok(verification)
    }
}
