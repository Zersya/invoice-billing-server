use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobQueue {
    pub id: i32,
    pub job_type: String,
    pub job_data: Option<Value>,
    pub job_schedule_id: Option<i32>,
    pub priority: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl JobQueue {
    pub async fn create(
        db: &sqlx::PgPool,
        job_type: &str,
        job_data: Option<Value>,
        job_schedule_id: Option<i32>,
        priority: i32,
        status: &str,
    ) -> Result<JobQueue, sqlx::Error> {
        let job_queue = sqlx::query_as!(
            JobQueue,
            r#"
            INSERT INTO job_queues (job_type, job_data, job_schedule_id, priority, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            job_type,
            job_data,
            job_schedule_id,
            priority,
            status
        )
        .fetch_one(db)
        .await?;

        Ok(job_queue)
    }

    pub async fn update_status(
        db: &sqlx::PgPool,
        id: &i32,
        status: &str,
    ) -> Result<JobQueue, sqlx::Error> {
        let job_queue = sqlx::query_as!(
            JobQueue,
            r#"
            UPDATE job_queues
            SET status = $2
            WHERE id = $1
            RETURNING *
            "#,
            id,
            status
        )
        .fetch_one(db)
        .await?;

        Ok(job_queue)
    }

    pub async fn get_top_priority_job(db: &sqlx::PgPool) -> Result<JobQueue, sqlx::Error> {
        let job_queue = sqlx::query_as!(
            JobQueue,
            r#"
            SELECT * FROM job_queues
            WHERE status = 'pending' OR status = 'failed' OR status = 'in_progress'
            ORDER BY priority ASC, created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_one(db)
        .await?;

        Ok(job_queue)
    }
}
