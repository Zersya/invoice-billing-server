use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub price: i32,
    pub tax: f32,
    pub discount: f32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_by: Uuid,
    pub invoice_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct SimpleItem {
    pub id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub price: i32,
    pub tax: f32,
    pub discount: f32,
}

impl Item {

    pub async fn create(
        db: &sqlx::PgPool,
        description: &str,
        quantity: &i32,
        price: &i32,
        tax: &f32,
        discount: &f32,
        created_by: &Uuid,
        invoice_id: &Uuid,
    ) -> Result<Item, sqlx::Error> {
        let item = sqlx::query_as!(
            Item,
            r#"
            INSERT INTO items (description, quantity, price, tax, discount, created_by, invoice_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            description,
            quantity,
            price,
            tax,
            discount,
            created_by,
            invoice_id,
        )
        .fetch_one(db)
        .await?;

        Ok(item)
    }

    pub async fn update(
        db: &sqlx::PgPool,
        id: &Uuid,
        description: &str,
        quantity: &i32,
        price: &i32,
        tax: &f32,
        discount: &f32,
        created_by: &Uuid,
        invoice_id: &Uuid,
    ) -> Result<Item, sqlx::Error> {
        let item = sqlx::query_as!(
            Item,
            r#"
            UPDATE items
            SET description = $1, quantity = $2, price = $3, tax = $4, discount = $5, created_by = $6, invoice_id = $7
            WHERE id = $8
            RETURNING *
            "#,
            description,
            quantity,
            price,
            tax,
            discount,
            created_by,
            invoice_id,
            id,
        )
        .fetch_one(db)
        .await?;

        Ok(item)
    }

    pub async fn delete (
        db: &sqlx::PgPool,
        id: &Uuid,
        invoice_id: &Uuid,
    ) -> Result<Item, sqlx::Error> {
        let item = sqlx::query_as!(
            Item,
            r#"
            DELETE FROM items
            WHERE id = $1 AND invoice_id = $2
            RETURNING *
            "#,
            id,
            invoice_id,
        )
        .fetch_one(db)
        .await?;

        Ok(item)
    }
}
