use std::ops::Add;

use axum::http::HeaderValue;
use chrono::{Duration, Utc};
use cron::Schedule;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::Errors,
    models::{
        customer_contact_channel::CustomerContactChannel, invoice::Invoice, job_queue::JobQueue,
        job_schedule::JobSchedule,
    },
    repositories::invoice::send_invoice_to_xendit,
};

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
                return Err(Errors::new(&[(
                    "whatsapp_send_message",
                    "Failed to send message",
                )]));
            }
        };

        return Ok(());
    }

    Err(Errors::new(&[(
        "whatsapp_send_message",
        "Datetime is not yet reached",
    )]))
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

        let is_queue_empty =
            match JobQueue::get_queue_not_completed_by_schedule_id(&pool, job_schedule_id).await {
                Ok(job_queues) => job_queues.len() == 0,
                Err(_) => false,
            };

        if !is_queue_empty {
            continue;
        }

        if job_schedule.job_type == "send_invoice" {
            let job_data = job_schedule.job_data.clone().unwrap();
            let invoice_id = job_data["invoice_id"].as_str().unwrap();
            let invoice_id = Uuid::parse_str(invoice_id).unwrap();

            let invoice = match Invoice::get_by_id(&pool, &invoice_id).await {
                Ok(invoice) => invoice,
                Err(_) => {
                    return;
                }
            };

            let result = match send_invoice_to_xendit(
                &invoice.invoice_number,
                &invoice.total_amount,
                &invoice.to_string(),
            )
            .await
            {
                Ok(payload) => payload,
                Err(_) => {
                    return;
                }
            };

            match Invoice::update_xendit_invoice_payload(&pool, &invoice.id, &result).await {
                Ok(invoice) => invoice,
                Err(_) => {
                    return;
                }
            };
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

pub async fn prepare_invoice_via_channels(
    pool: &PgPool,
    job_data: Value,
    schedule: &Schedule,
) -> Result<(), Errors> {
    let invoice_id = match job_data["invoice_id"].as_str() {
        Some(invoice_id) => uuid::Uuid::parse_str(invoice_id).unwrap(),
        None => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let customer_id = match job_data["customer_id"].as_str() {
        Some(phone_number) => uuid::Uuid::parse_str(phone_number).unwrap(),
        None => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let merchant_id = match job_data["merchant_id"].as_str() {
        Some(merchant_id) => uuid::Uuid::parse_str(merchant_id).unwrap(),
        None => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let total_amount = match job_data["total_amount"].as_i64() {
        Some(total_amount) => total_amount,
        None => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let customer_contact_channels =
        match CustomerContactChannel::get_customer_contact_channels_by_customer_and_merchant(
            &pool,
            &customer_id,
            &merchant_id,
        )
        .await
        {
            Ok(customer_contact_channels) => customer_contact_channels,
            Err(_) => {
                return Err(Errors::new(&[(
                    "prepare_invoice",
                    "Failed to prepare invoice",
                )]));
            }
        };

    // This code finds the whatsapp contact channel, if it exists.
    let whatsapp_contact_channel = match customer_contact_channels
        .iter()
        .find(|contact_channel| contact_channel.name == "whatsapp")
    {
        Some(whatsapp_contact_channel) => whatsapp_contact_channel,
        None => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let invoice = match Invoice::get_by_id(&pool, &invoice_id).await {
        Ok(invoice) => invoice,
        Err(_) => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let xendit_invoice_payload = invoice.xendit_invoice_payload.unwrap();
    let invoice_url = xendit_invoice_payload["invoice_url"].as_str().unwrap();

    let job_schedule =
        match JobSchedule::get_by_job_data_json_by_invoice_id(&pool, &invoice.id.to_string().as_str()).await {
            Ok(job_schedule) => job_schedule,
            Err(_) => {
                return Err(Errors::new(&[(
                    "prepare_invoice",
                    "Failed to prepare invoice",
                )]));
            }
        };

    let repeat_interval = if job_schedule.repeat_interval.is_some() {
        job_schedule.repeat_interval.unwrap()
    } else {
        2
    };

    let now = Utc::now();
    let due_time = &now.add(Duration::seconds(repeat_interval));
    let _ = now.signed_duration_since(*due_time).num_days();
    let due_time = format!("{}", due_time.format("%d/%m/%Y - %H:%M"));

    
    let total_amount = format!("Rp{:.2}", total_amount);

    match whatsapp_send_message(
        whatsapp_contact_channel.value.as_str(),
        format!(
            "Please make a payment of *{}* to avoid incurring late fees. The *payment link* is {} and the *due date* is {}.",
            total_amount,
            invoice_url,
            due_time
        )
        .as_str(),
        &schedule,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => {
            return Err(Errors::new(&[(
                "prepare_invoice",
                "Failed to prepare invoice",
            )]));
        }
    }
}
