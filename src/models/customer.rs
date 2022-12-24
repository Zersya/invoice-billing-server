use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub merchant_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Debug)]
pub struct CustomerWithContactChannels {
    pub id: Uuid,
    pub name: String,
    pub merchant_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub contact_channel_id: Uuid,
    pub contact_channel_value: String,
    pub contact_channel_name: String,
}

impl Customer {
    pub async fn create(
        db: &sqlx::PgPool,
        name: &String,
        merchant_id: &Uuid,
    ) -> Result<Customer, sqlx::Error> {
        let customer = sqlx::query_as!(
            Customer,
            r#"
            INSERT INTO customers (name, merchant_id)
            VALUES ($1, $2)
            RETURNING *
            "#,
            name,
            merchant_id
        )
        .fetch_one(db)
        .await?;

        Ok(customer)
    }

    pub async fn create_using_transaction(
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: &String,
        merchant_id: &Uuid,
    ) -> Result<Customer, sqlx::Error> {
        let customer = sqlx::query_as!(
            Customer,
            r#"
            INSERT INTO customers (name, merchant_id)
            VALUES ($1, $2)
            RETURNING *
            "#,
            name,
            merchant_id
        )
        .fetch_one(db)
        .await?;

        Ok(customer)
    }

    pub async fn update(
        db: &sqlx::PgPool,
        id: &Uuid,
        name: &String,
        merchant_id: &Uuid,
    ) -> Result<Customer, sqlx::Error> {
        let customer = sqlx::query_as!(
            Customer,
            r#"
            UPDATE customers
            SET name = $1
            WHERE id = $2 AND merchant_id = $3 AND deleted_at IS NULL
            RETURNING *
            "#,
            name,
            id,
            merchant_id
        )
        .fetch_one(db)
        .await?;

        Ok(customer)
    }

    pub async fn get_by_merchat_user_id(
        db: &sqlx::PgPool,
        user_id: &Uuid,
    ) -> Result<Vec<Customer>, sqlx::Error> {
        let customers = sqlx::query_as!(
            Customer,
            r#"
            SELECT customers.*
            FROM customers
            INNER JOIN merchants ON merchants.id = customers.merchant_id
            WHERE merchants.user_id = $1 AND customers.deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_all(db)
        .await?;

        Ok(customers)
    }

    pub async fn get_by_merchant_id(
        db: &sqlx::PgPool,
        merchant_id: &Uuid,
    ) -> Result<Vec<CustomerWithContactChannels>, sqlx::Error> {
        let customers = sqlx::query_as!(
            CustomerWithContactChannels,
            r#"
            SELECT
                customers.*, 
                customer_contact_channels.contact_channel_id as contact_channel_id, 
                customer_contact_channels.value as contact_channel_value, 
                contact_channels.name as contact_channel_name
            FROM
                customers
                INNER JOIN customer_contact_channels ON customer_contact_channels.customer_id = customers.id
                INNER JOIN contact_channels ON contact_channels.id = customer_contact_channels.contact_channel_id
            WHERE
                merchant_id = $1
                AND customers.deleted_at IS NULL
            "#,
            merchant_id
        )
        .fetch_all(db)
        .await?;

        Ok(customers)
    }

    pub async fn get_by_id(
        db: &sqlx::PgPool,
        id: Uuid,
        merchant_id: &Uuid,
    ) -> Result<Customer, sqlx::Error> {
        let customer = sqlx::query_as!(
            Customer,
            r#"
            SELECT *
            FROM customers
            WHERE id = $1 AND merchant_id = $2 AND deleted_at IS NULL
            "#,
            id,
            merchant_id
        )
        .fetch_one(db)
        .await?;

        Ok(customer)
    }

    pub async fn delete(
        db: &sqlx::PgPool,
        id: &Uuid,
        merchant_id: &Uuid,
    ) -> Result<Customer, sqlx::Error> {
        let customer = sqlx::query_as!(
            Customer,
            r#"
            UPDATE customers
            SET deleted_at = NOW()
            WHERE id = $1 AND merchant_id = $2 AND deleted_at IS NULL
            RETURNING *
            "#,
            id,
            merchant_id
        )
        .fetch_one(db)
        .await?;

        Ok(customer)
    }
}
