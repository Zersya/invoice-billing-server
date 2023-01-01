use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tester {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stage: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Tester {
    pub async fn get_by_user_id(
        db: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Tester, sqlx::Error> {
        let tester = sqlx::query_as!(
            Tester,
            r#"
            SELECT * FROM testers
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(tester)
    }
    
}
