use axum::http::HeaderValue;
use chrono::Utc;
use cron::Schedule;
use sqlx::PgPool;

use crate::{errors::Errors, models::{job_schedule::JobSchedule, job_queue::JobQueue}};


pub async fn whatsapp_send_message(
    phone_number: &str,
    message: &str,
    schedule: &Schedule,
) -> Result<(), Errors> {
    let client = reqwest::Client::new();

    let now = Utc::now();

    let schedule = schedule.upcoming(Utc).take(1);
    let datetime = schedule.last().unwrap();

    if datetime >= now {
        println!("Sending message... {} {}", phone_number, message);
        let host = std::env::var("WHATSAPP_BASE_URL").unwrap();
        let whatsapp_api_key = std::env::var("WHATSAPP_API_KEY").unwrap();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            "x-whatsapp-api-key",
            HeaderValue::from_str(&whatsapp_api_key.as_str()).unwrap(),
        );

        match client
            .post(format!("{}/api/send", host))
            .headers(headers)
            .query(&[("number", phone_number), ("message", message)])
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => {
                return Err(Errors::new(&[("whatsapp_send_message", "Failed to send message")]));
            }
        };

        return Ok(());

    }

    Err(Errors::new(&[("whatsapp_send_message", "Datetime is not yet reached")]))

}

pub async fn set_job_schedule_to_queue(pool: PgPool) {

    let job_schedules = match JobSchedule::get_scheduled_jobs(&pool).await {
        Ok(job_schedules) => job_schedules,
        Err(_) => {
            return;
        }
    };

    for job_schedule in job_schedules {
        let job_schedule_id = job_schedule.id;

        match JobSchedule::update_status(&pool, job_schedule_id, "pending").await {
            Ok(_) => (),
            Err(_) => {
                return;
            }
        };

        let priority = match job_schedule.job_type.as_str() {
            "send_invoice" => 0,
            "send_reminder" => 1,
            _ => 10,
        };

        let is_queue_empty = match JobQueue::get_queue_not_completed_by_schedule_id(&pool, job_schedule_id).await {
            Ok(job_queues) => {
                job_queues.len() == 0
            }
            Err(_) => {
                false
            }
        };

        if !is_queue_empty {
            continue;
        }

        match JobQueue::create(
            &pool,
            &job_schedule.job_type,
            job_schedule.job_data,
            Some(job_schedule_id),
            priority,
            "pending",
        )
        .await
        {
            Ok(_) => (),
            Err(_) => {
                return;
            }
        }
    }
}
