use std::{ops::Add, time::Duration};

use cron::Schedule;
use sqlx::PgPool;
use tokio::time::interval;

use crate::models::{
    customer_contact_channel::CustomerContactChannel, job_queue::JobQueue,
    job_schedule::JobSchedule,
};

use super::actions::{
    prepare_invoice_via_channels, set_job_schedule_to_queue, whatsapp_send_message,
};

pub async fn spawn_job_queue(pool: PgPool, schedule: Schedule) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let job = match JobQueue::get_top_priority_job(&pool).await {
                Ok(job) => job,
                Err(_) => {
                    continue;
                }
            };

            if job.job_schedule_id.is_some() {
                let job_schedule =
                    JobSchedule::get_schedule_by_id(&pool, job.job_schedule_id.unwrap())
                        .await
                        .expect("Failed to get schedule by id");

                JobSchedule::update_status(&pool, job.job_schedule_id.unwrap(), "in_progress")
                    .await
                    .expect("Failed to update status");

                if job_schedule.repeat_count.is_some() && job_schedule.repeat_count.unwrap() > 0 {
                    let repeat_count = job_schedule.repeat_count.unwrap();

                    if job_schedule.repeat_interval.is_some() {
                        let repeat_interval = job_schedule.repeat_interval.unwrap();

                        let new_run_at = job_schedule
                            .run_at
                            .add(chrono::Duration::seconds(repeat_interval));

                        JobSchedule::update_run_at(&pool, job_schedule.id, &new_run_at)
                            .await
                            .expect("Failed to update run at");
                    }

                    JobSchedule::update_repeat_count(
                        &pool,
                        job.job_schedule_id.unwrap(),
                        repeat_count - 1,
                    )
                    .await
                    .expect("Failed to update repeat count");
                } else {
                    JobSchedule::update_status(&pool, job.job_schedule_id.unwrap(), "completed")
                        .await
                        .expect("Failed to update status");
                }
            }

            JobQueue::update_status(&pool, &job.id, "in_progress")
                .await
                .expect("Failed to update status");

            if job.job_data.is_none() {
                JobQueue::update_status(&pool, &job.id, "failed")
                    .await
                    .expect("Failed to update status");

                continue;
            }

            let job_data = match serde_json::from_value::<serde_json::Value>(job.job_data.unwrap())
            {
                Ok(job_data) => job_data,
                Err(_) => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update status");

                    continue;
                }
            };

            match prepare_invoice_via_channels(&pool, job_data, &schedule).await {
                Ok(_) => {
                    JobQueue::update_status(&pool, &job.id, "completed")
                        .await
                        .expect("Failed to update status");

                    if job.job_schedule_id.is_some() {
                        let job_schedule =
                            JobSchedule::get_schedule_by_id(&pool, job.job_schedule_id.unwrap())
                                .await
                                .expect("Failed to get schedule by id");

                        if job_schedule.repeat_count.is_some()
                            && job_schedule.repeat_count.unwrap() == 0
                        {
                            JobSchedule::update_status(
                                &pool,
                                job.job_schedule_id.unwrap(),
                                "completed",
                            )
                            .await
                            .expect("Failed to update status");
                        }
                    }
                }
                Err(_) => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update status");
                }
            }
        }
    });
}

pub async fn spawn_set_job_schedule_to_queue(pool: PgPool) {
    tokio::spawn(async move {
        // Use an interval to perform the check at regular intervals.
        let mut interval = interval(Duration::from_secs(15));

        loop {
            interval.tick().await;
            set_job_schedule_to_queue(pool.clone()).await;
        }
    });
}
