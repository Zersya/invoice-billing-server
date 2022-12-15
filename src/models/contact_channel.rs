use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContactChannel {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
pub struct SimpleContactChannel {
    pub id: Uuid,
    pub name: String,
}

impl ContactChannel {
    pub async fn get_all (
        db: &sqlx::PgPool,
    ) -> Result<Vec<ContactChannel>, sqlx::Error> {
        let contact_channels = sqlx::query_as!(
            ContactChannel,
            r#"
            SELECT *
            FROM contact_channels
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(contact_channels)
    }
}