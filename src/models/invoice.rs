use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::job_schedule::JobSchedule;

#[derive(Serialize, Deserialize, Debug)]
pub struct Invoice {
    pub id: Uuid,
    pub invoice_number: String,
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
    pub xendit_invoice_payload: Option<Value>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct InvoiceWithCustomer {
    pub id: Uuid,
    pub invoice_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub total_amount: i32,
    pub invoice_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub job_schedule: Option<Value>,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl Invoice {
    pub async fn create(
        db: &sqlx::PgPool,
        invoice_number: &str,
        customer_id: &Uuid,
        merchant_id: &Uuid,
        amount: &i32,
        total_amount: &i32,
        tax_amount: &i32,
        tax_rate: &i32,
        invoice_date: &NaiveDateTime,
        created_by: &Uuid,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<Invoice, sqlx::Error> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
            INSERT INTO invoices (invoice_number, customer_id, merchant_id, amount, total_amount, tax_amount, tax_rate, invoice_date, created_by, title, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
            invoice_number,
            customer_id,
            merchant_id,
            amount,
            total_amount,
            tax_amount,
            tax_rate,
            invoice_date,
            created_by,
            title,
            description
        )
        .fetch_one(db)
        .await?;

        Ok(invoice)
    }

    pub async fn update_xendit_invoice_payload(
        db: &sqlx::PgPool,
        invoice_id: &Uuid,
        xendit_invoice_payload: &Value,
    ) -> Result<Invoice, sqlx::Error> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
            UPDATE invoices
            SET xendit_invoice_payload = $1
            WHERE id = $2
            RETURNING *
            "#,
            xendit_invoice_payload,
            invoice_id
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
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<Invoice, sqlx::Error> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
            INSERT INTO invoices (customer_id, merchant_id, amount, total_amount, tax_amount, tax_rate, invoice_date, created_by, title, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            customer_id,
            merchant_id,
            amount,
            total_amount,
            tax_amount,
            tax_rate,
            invoice_date,
            created_by,
            title,
            description
        )
        .fetch_one(db)
        .await?;

        Ok(invoice)
    }

    pub async fn get_by_merchat_user_id(
        db: &sqlx::PgPool,
        user_id: &Uuid,
    ) -> Result<Vec<InvoiceWithCustomer>, sqlx::Error> {
        let invoices = sqlx::query_as!(
            InvoiceWithCustomer,
            r#"
            SELECT 
                invoices.id, 
                invoices.invoice_number, 
                invoices.customer_id, 
                customers.name as customer_name, 
                invoices.total_amount, 
                invoices.invoice_date, 
                invoices.created_at, 
                row_to_json(job_schedules) as job_schedule,
                invoices.title,
                invoices.description 
            FROM invoices
                INNER JOIN merchants ON merchants.id = invoices.merchant_id
                INNER JOIN users ON users.id = merchants.user_id
                INNER JOIN customers ON customers.id = invoices.customer_id
                LEFT JOIN job_schedules ON job_schedules.job_data->>'invoice_id' = invoices.id::text
            WHERE users.id = $1
            ORDER BY invoices.created_at DESC
            "#,
            user_id
        )
        .fetch_all(db)
        .await?;

        Ok(invoices)
    }

    pub async fn get_by_merchant_id(
        db: &sqlx::PgPool,
        merchant_id: &Uuid,
    ) -> Result<Vec<InvoiceWithCustomer>, sqlx::Error> {
        let invoices = sqlx::query_as!(
            InvoiceWithCustomer,
            r#"
            SELECT 
                invoices.id, 
                invoices.invoice_number, 
                invoices.customer_id, 
                customers.name as customer_name, 
                invoices.total_amount, 
                invoices.invoice_date, 
                invoices.created_at, 
                row_to_json(job_schedules) as job_schedule,
                invoices.title,
                invoices.description
            FROM invoices
                INNER JOIN customers ON customers.id = invoices.customer_id
                LEFT JOIN job_schedules ON job_schedules.job_data->>'invoice_id' = invoices.id::text
            WHERE invoices.merchant_id = $1
            ORDER BY invoices.created_at DESC
            "#,
            merchant_id
        )
        .fetch_all(db)
        .await?;

        Ok(invoices)
    }

    pub async fn get_by_id(db: &sqlx::PgPool, id: &Uuid) -> Result<Invoice, sqlx::Error> {
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

    pub fn to_string(&self) -> String {
        format!(
            "customer_id: {}, total_amount: {}, invoice_date: {}",
            self.customer_id, self.total_amount, self.invoice_date
        )
    }
}
