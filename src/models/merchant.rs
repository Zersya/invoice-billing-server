use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Merchant {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Merchant {
    pub async fn create(
        db: &sqlx::PgPool,
        name: &String,
        description: &String,
        user_id: Uuid,
    ) -> Result<Merchant, sqlx::Error> {
        let merchant = sqlx::query_as!(
            Merchant,
            r#"
            INSERT INTO merchants (name, description, user_id)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            name,
            description,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(merchant)
    }

    pub async fn update(
        db: &sqlx::PgPool,
        id: Uuid,
        name: &String,
        description: &String,
        user_id: Uuid,
    ) -> Result<Merchant, sqlx::Error> {
        let merchant = sqlx::query_as!(
            Merchant,
            r#"
            UPDATE merchants
            SET name = $1, description = $2
            WHERE id = $3 AND user_id = $4 AND deleted_at IS NULL
            RETURNING *
            "#,
            name,
            description,
            id,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(merchant)
    }

    pub async fn delete (
        db: &sqlx::PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Merchant, sqlx::Error> {
        let merchant = sqlx::query_as!(
            Merchant,
            r#"
            UPDATE merchants
            SET deleted_at = NOW()
            WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
            RETURNING *
            "#,
            id,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(merchant)
    }

    pub async fn get_by_user_id(
        db: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Merchant>, sqlx::Error> {
        let merchants = sqlx::query_as!(
            Merchant,
            r#"
            SELECT * FROM merchants
            WHERE user_id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_all(db)
        .await?;

        Ok(merchants)
    }
}