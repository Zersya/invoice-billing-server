use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Invoice {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Uuid,
    pub amount: i32,
    pub total_amount: i32,
    pub tax_amount: i32,
    pub tax_rate: i32,
    pub invoice_date: NaiveDateTime,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Invoice {
    pub async fn create(
        db: &sqlx::PgPool,
        customer_id: &Uuid,
        merchant_id: &Uuid,
        amount: &i32,
        total_amount: &i32,
        tax_amount: &i32,
        tax_rate: &i32,
        invoice_date: &NaiveDateTime,
        created_by: &Uuid,
    ) -> Result<Invoice, sqlx::Error> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
            INSERT INTO invoices (customer_id, merchant_id, amount, total_amount, tax_amount, tax_rate, invoice_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            customer_id,
            merchant_id,
            amount,
            total_amount,
            tax_amount,
            tax_rate,
            invoice_date,
            created_by
        )
        .fetch_one(db)
        .await?;

        Ok(invoice)
    }

    pub async fn create_using_transaction(
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_id: &Uuid,
        merchant_id: &Uuid,
        amount: &i32,
        total_amount: &i32,
        tax_amount: &i32,
        tax_rate: &i32,
        invoice_date: &NaiveDateTime,
        created_by: &Uuid,
    ) -> Result<Invoice, sqlx::Error> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
            INSERT INTO invoices (customer_id, merchant_id, amount, total_amount, tax_amount, tax_rate, invoice_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            customer_id,
            merchant_id,
            amount,
            total_amount,
            tax_amount,
            tax_rate,
            invoice_date,
            created_by
        )
        .fetch_one(db)
        .await?;

        Ok(invoice)
    }

    pub async fn get_by_merchat_user_id(
        db: &sqlx::PgPool,
        user_id: &Uuid,
    ) -> Result<Vec<Invoice>, sqlx::Error> {
        let invoices = sqlx::query_as!(
            Invoice,
            r#"
            SELECT invoices.*
            FROM invoices
            INNER JOIN merchants ON merchants.id = invoices.merchant_id
            INNER JOIN users ON users.id = merchants.user_id
            WHERE users.id = $1
            "#,
            user_id
        )
        .fetch_all(db)
        .await?;

        Ok(invoices)
    }

    pub async fn get_by_id (
        db: &sqlx::PgPool,
        id: &Uuid,
    ) -> Result<Invoice, sqlx::Error> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
            SELECT *
            FROM invoices
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(invoice)
    }
}
