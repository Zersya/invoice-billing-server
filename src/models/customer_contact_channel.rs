use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomerContactChannel {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub contact_channel_id: Uuid,
    pub value: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomerContactChannelWithContactChannel {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub value: String,
    pub name: String,
}

impl CustomerContactChannel {
    pub async fn create_using_transaction(
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_id: &Uuid,
        contact_channel_id: &Uuid,
        value: &String,
    ) -> Result<CustomerContactChannel, sqlx::Error> {
        let customer_contact_channel = sqlx::query_as!(
            CustomerContactChannel,
            r#"
            INSERT INTO customer_contact_channels (customer_id, contact_channel_id, value)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            customer_id,
            contact_channel_id,
            value
        )
        .fetch_one(db)
        .await?;

        Ok(customer_contact_channel)
    }

    pub async fn get_customer_contact_channels_by_customer_and_merchant(
        db: &sqlx::PgPool,
        customer_id: &Uuid,
        merchant_id: &Uuid,
    ) -> Result<Vec<CustomerContactChannelWithContactChannel>, sqlx::Error> {
        let customer_contact_channels = sqlx::query_as!(
            CustomerContactChannelWithContactChannel,
            r#"
            SELECT
                a.id,
                a.customer_id,
                a.value,
                c.name
            FROM
                customer_contact_channels a
                LEFT JOIN customers b ON b.id = a.customer_id
                LEFT JOIN contact_channels c ON c.id = a.contact_channel_id
            WHERE 
                a.customer_id = $1 AND b.merchant_id = $2 AND a.deleted_at IS NULL
            GROUP BY
                a.id, c.name
            "#,
            customer_id,
            merchant_id
        )
        .fetch_all(db)
        .await?;

        Ok(customer_contact_channels)
    }
}
