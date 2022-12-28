use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobSchedule {
    pub id: i32,
    pub job_type: String,
    pub job_data: Option<Value>,
    pub run_at: NaiveDateTime,
    pub repeat_interval: Option<i64>,
    pub repeat_count: Option<i32>,
    pub dependencies: Option<String>,
    pub status: String,
    pub retry_count: Option<i32>,
    pub retry_interval: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl JobSchedule {
    pub async fn create(
        db: &sqlx::PgPool,
        job_type: &str,
        job_data: Option<Value>,
        run_at: &NaiveDateTime,
        repeat_interval: Option<i64>,
        repeat_count: Option<i32>,
        dependencies: Option<String>,
        status: &str,
        retry_count: Option<i32>,
        retry_interval: Option<i32>,
    ) -> Result<JobSchedule, sqlx::Error> {
        let job_schedule = sqlx::query_as!(
            JobSchedule,
            r#"
            INSERT INTO job_schedules (job_type, job_data, run_at, repeat_interval, repeat_count, dependencies, status, retry_count, retry_interval)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            job_type,
            job_data,
            run_at,
            repeat_interval,
            repeat_count,
            dependencies,
            status,
            retry_count,
            retry_interval
        )
        .fetch_one(db)
        .await?;

        Ok(job_schedule)
    }

    pub async fn get_schedule_by_id(db: &sqlx::PgPool, id: i32) -> Result<JobSchedule, sqlx::Error> {
        let job_schedule = sqlx::query_as!(
            JobSchedule,
            r#"
            SELECT * FROM job_schedules
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(job_schedule)
    }

    pub async fn get_scheduled_jobs(db: &sqlx::PgPool) -> Result<Vec<JobSchedule>, sqlx::Error> {
        let job_schedules = sqlx::query_as!(
            JobSchedule,
            r#"
            SELECT * FROM job_schedules
            WHERE (status = 'scheduled' OR status = 'pending' OR status = 'in_progress') AND run_at <= now()
            "#
        )
        .fetch_all(db)
        .await?;

        Ok(job_schedules)
    }

    pub async fn update_status(
        db: &sqlx::PgPool,
        id: i32,
        status: &str,
    ) -> Result<JobSchedule, sqlx::Error> {
        let job_schedule = sqlx::query_as!(
            JobSchedule,
            r#"
            UPDATE job_schedules
            SET status = $1
            WHERE id = $2
            RETURNING *
            "#,
            status,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(job_schedule)
    }

    pub async fn update_repeat_count(
        db: &sqlx::PgPool,
        id: i32,
        repeat_count: i32,
    ) -> Result<JobSchedule, sqlx::Error> {
        let job_schedule = sqlx::query_as!(
            JobSchedule,
            r#"
            UPDATE job_schedules
            SET repeat_count = $1
            WHERE id = $2
            RETURNING *
            "#,
            repeat_count,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(job_schedule)
    }

    pub async fn update_run_at(
        db: &sqlx::PgPool,
        id: i32,
        run_at: &NaiveDateTime,
    ) -> Result<JobSchedule, sqlx::Error> {
        let job_schedule = sqlx::query_as!(
            JobSchedule,
            r#"
            UPDATE job_schedules
            SET run_at = $1
            WHERE id = $2
            RETURNING *
            "#,
            run_at,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(job_schedule)
    }

    pub async fn get_by_job_data_json_by_invoice_id(
        db: &sqlx::PgPool,
        invoice_id: &str,
    ) -> Result<JobSchedule, sqlx::Error> {
        let job_schedules = sqlx::query_as!(
            JobSchedule,
            r#"
            SELECT * FROM job_schedules
            WHERE job_data->>'invoice_id' = $1
            LIMIT 1
            "#,
            invoice_id
        )
        .fetch_one(db)
        .await?;

        Ok(job_schedules)
    }

    pub async fn get_by_job_data_json_by_customer_id(
        db: &sqlx::PgPool,
        customer_id: &str,
    ) -> Result<Vec<JobSchedule>, sqlx::Error> {
        let job_schedules = sqlx::query_as!(
            JobSchedule,
            r#"
            SELECT * FROM job_schedules
            WHERE job_data->>'customer_id' = $1
            "#,
            customer_id
        )
        .fetch_all(db)
        .await?;

        Ok(job_schedules)
    }

    pub async fn get_by_job_data_json_by_merchant_id(
        db: &sqlx::PgPool,
        merchant_id: &str,
    ) -> Result<Vec<JobSchedule>, sqlx::Error> {
        let job_schedules = sqlx::query_as!(
            JobSchedule,
            r#"
            SELECT * FROM job_schedules
            WHERE job_data->>'merchant_id' = $1
            "#,
            merchant_id
        )
        .fetch_all(db)
        .await?;

        Ok(job_schedules)
    }
}
