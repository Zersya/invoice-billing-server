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
    pub address: Option<String>,
    pub phone_country_code: Option<String>,
    pub phone_number: Option<String>,
    pub tax: Option<f32>
}

impl Merchant {
    pub async fn create(
        db: &sqlx::PgPool,
        name: &String,
        description: &String,
        user_id: &Uuid,
        address: Option<String>,
        phone_country_code: Option<String>,
        phone_number: Option<String>,
        tax: Option<f32>
    ) -> Result<Merchant, sqlx::Error> {
        let merchant = sqlx::query_as!(
            Merchant,
            r#"
            INSERT INTO merchants (name, description, user_id, address, phone_country_code, phone_number, tax)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            name,
            description,
            user_id,
            address,
            phone_country_code,
            phone_number,
            tax
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
        user_id: &Uuid,
        address: Option<String>,
        phone_country_code: Option<String>,
        phone_number: Option<String>,
        tax: Option<f32>
    ) -> Result<Merchant, sqlx::Error> {
        let merchant = sqlx::query_as!(
            Merchant,
            r#"
            UPDATE merchants
            SET name = $1, description = $2, address = $3, phone_country_code = $4, phone_number = $5, tax = $6
            WHERE id = $7 AND user_id = $8 AND deleted_at IS NULL
            RETURNING *
            "#,
            name,
            description,
            address,
            phone_country_code,
            phone_number,
            tax,
            id,
            user_id,
        )
        .fetch_one(db)
        .await?;

        Ok(merchant)
    }

    pub async fn delete(
        db: &sqlx::PgPool,
        id: Uuid,
        user_id: &Uuid,
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
        user_id: &Uuid,
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

    pub async fn get_by_id(db: &sqlx::PgPool, id: Uuid) -> Result<Merchant, sqlx::Error> {
        let merchant = sqlx::query_as!(
            Merchant,
            r#"
            SELECT * FROM merchants
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id,
        )
        .fetch_one(db)
        .await?;

        Ok(merchant)
    }
}
