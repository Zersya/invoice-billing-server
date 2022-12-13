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

impl CustomerContactChannel {
    pub async fn create_using_transaction (
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
}