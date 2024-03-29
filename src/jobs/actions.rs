use std::ops::Add;

use chrono::{Duration, Utc};
use cron::Schedule;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use rand::Rng;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::Errors,
    models::{
        customer_contact_channel::CustomerContactChannel, invoice::Invoice, job_queue::JobQueue,
        job_schedule::JobSchedule,
    },
    repositories::{
        invoice::send_invoice_to_xendit, telegram::telegram_send_message,
        whatsapp::whatsapp_send_message,
    },
};

pub async fn set_job_schedule_to_queue(pool: PgPool) {
    let job_schedules = match JobSchedule::get_scheduled_jobs(&pool).await {
        Ok(job_schedules) => job_schedules,
        Err(_) => {
            return;
        }
    };

    for mut job_schedule in job_schedules {
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
            let job_data = match job_schedule.job_data {
                Some(job_data) => job_data,
                None => {
                    return;
                }
            };

            job_schedule.job_data =
                match set_job_schedule_send_invoice(&pool, job_data, job_schedule_id).await {
                    Ok(job_data) => Some(job_data),
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

pub async fn prepare_via_channels(
    pool: &PgPool,
    job_schedule: &JobSchedule,
    schedule: &Schedule,
) -> Result<(), Errors> {
    let job_data = match &job_schedule.job_data {
        Some(job_data) => job_data,
        None => {
            return Err(Errors::new(&[(
                "prepare_via_channels",
                "unable to get job data",
            )]));
        }
    };

    let customer_id = match job_data["customer_id"].as_str() {
        Some(phone_number) => uuid::Uuid::parse_str(phone_number).unwrap(),
        None => {
            return Err(Errors::new(&[(
                "prepare_via_channels",
                "Failed to prepare invoice",
            )]));
        }
    };

    let merchant_id = match job_data["merchant_id"].as_str() {
        Some(merchant_id) => uuid::Uuid::parse_str(merchant_id).unwrap(),
        None => {
            return Err(Errors::new(&[(
                "prepare_via_channels",
                "Failed to prepare invoice",
            )]));
        }
    };

    let merchant_name = match job_data["merchant_name"].as_str() {
        Some(merchant_name) => merchant_name.to_string(),
        None => {
            return Err(Errors::new(&[(
                "prepare_via_channels",
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
                    "prepare_via_channels",
                    "Failed to prepare invoice",
                )]));
            }
        };

    let mut message = String::new();

    if job_schedule.job_type == "send_invoice" {
        message = match message_builder_invoice(&pool, job_data.clone(), &merchant_name).await {
            Ok(message) => message,
            Err(_) => {
                return Err(Errors::new(&[(
                    "prepare_via_channels",
                    "Failed to prepare invoice",
                )]));
            }
        };
    } else if job_schedule.job_type == "send_reminder" {
        message = match message_builder_reminder(job_data.clone(), &merchant_name) {
            Ok(message) => message,
            Err(_) => {
                return Err(Errors::new(&[(
                    "prepare_via_channels",
                    "Failed to prepare reminder",
                )]));
            }
        };
    };

    for contact_channel in customer_contact_channels.iter() {
        if contact_channel.name == "whatsapp" {
            let now = Utc::now();

            let schedule = schedule.upcoming(Utc).take(1);
            let datetime = schedule.last().unwrap();

            if datetime >= now {
                match whatsapp_send_message(contact_channel.value.as_str(), message.as_str()).await
                {
                    Ok(_) => (),
                    Err(_) => {
                        return Err(Errors::new(&[(
                            "prepare_via_channels",
                            "Failed to send message",
                        )]));
                    }
                }
            }
        } else if contact_channel.name == "email" {
            let email = Message::builder()
                .from("Reminder <hello@inving.co>".parse().unwrap())
                .to(contact_channel.value.parse().unwrap())
                .subject("Reminder")
                .body(message.clone())
                .unwrap();

            let password = std::env::var("EMAIL_SENDGRID_API_KEY").unwrap();
            let creds = Credentials::new("apikey".to_string(), password);

            let mailer = SmtpTransport::relay("smtp.sendgrid.net")
                .unwrap()
                .credentials(creds)
                .build();

            match mailer.send(&email) {
                Ok(_) => println!("Email sent successfully!"),
                Err(e) => panic!("Could not send email: {:?}", e),
            }
        } else if contact_channel.name == "telegram" {
            let chat_id = if contact_channel.additional_value.is_some() {
                contact_channel
                    .additional_value
                    .clone()
                    .unwrap()
                    .parse::<i64>()
                    .expect("format contact value invalid")
            } else {
                return Err(Errors::new(&[(
                    "no_additional_value",
                    "No additional value registered",
                )]));
            };

            match telegram_send_message(&chat_id, message.as_str()).await {
                Ok(_) => (),
                Err(_) => {
                    return Err(Errors::new(&[(
                        "prepare_via_channels",
                        "Failed to send message",
                    )]));
                }
            }
        }
    }

    Ok(())
}

// generate constant vector of messages
// generate random number between 0 to 9
// return random message from vector
fn generate_message() -> String {
    let messages = [
        "Hello {}, {} here, as a reminder, we ask that you please make a payment of *{}* to avoid any late fees. The payment can be made at the following link: {}. The due date for this payment is {}.",
        "Hello {}, {} here, to avoid incurring late fees, we request that you make a payment of *{}* as soon as possible. You can easily do so by following this payment link: {}. The deadline for this payment is {}.",
        "Hello {}, {} here, we strongly encourage you to make a payment of *{}* by the due date of {} to avoid late fees. You can make the payment by clicking on the following link: {}.",
        "Hello {}, {} here, to avoid being charged late fees, we request that you make a payment of *{}* by {}. You can access the payment link here: {}.",
        "Hello {}, {} here, please make a payment of *{}* by the due date of {} to avoid late fees. You can make the payment at the following link: {}.",
        "Hello {}, {} here, we request that you make a payment of *{}* as soon as possible to avoid any late fees. The payment link can be found here: {}. Please note that the payment is due on {}.",
        "Hello {}, {} here, to avoid late fees, we ask that you make a payment of *{}* by the due date of {}. You can make the payment using the following link: {}.",
        "Hello {}, {} here, as a reminder, a payment of *{}* is due on {} to avoid late fees. You can make the payment at the following link: {}.",
        "Hello {}, {} here, we request that you make a payment of *{}* by {} to avoid any late fees. The payment link is available here: {}.",
        "Hello {}, {} here, to avoid being charged late fees, we ask that you make a payment of *{}* as soon as possible. The payment link is provided here: {}. Please note that the payment is due on {}.",
    ];

    let random_number = rand::thread_rng().gen_range(0..10);
    messages[random_number].to_string()
}

async fn set_job_schedule_send_invoice(
    pool: &PgPool,
    job_data: Value,
    job_schedule_id: i32,
) -> Result<Value, Errors> {
    let invoice_id = job_data["invoice_id"].as_str().unwrap();
    let invoice_id = Uuid::parse_str(invoice_id).unwrap();

    let invoice = match Invoice::get_by_id(&pool, &invoice_id).await {
        Ok(invoice) => invoice,
        Err(_) => {
            return Err(Errors::new(&[("setup_invoice", "Failed to get invoice")]));
        }
    };

    // update invoice date to today
    let invoice_date = Utc::now().naive_utc();
    match Invoice::update_invoice_date(&pool, &invoice.id, &invoice_date).await {
        Ok(invoice) => invoice,
        Err(_) => {
            return Err(Errors::new(&[(
                "setup_invoice",
                "Failed to update invoice date",
            )]));
        }
    };

    let mut job_data = job_data;
    job_data["invoice_date"] = Value::String(invoice_date.to_string());

    match JobSchedule::update_job_data(&pool, job_schedule_id, &job_data).await {
        Ok(_) => (),
        Err(_) => {
            return Err(Errors::new(&[(
                "setup_invoice",
                "Failed to update job data",
            )]));
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
            return Err(Errors::new(&[(
                "setup_invoice",
                "Failed to send invoice to xendit",
            )]));
        }
    };

    match Invoice::update_xendit_invoice_payload(&pool, &invoice.id, &result).await {
        Ok(invoice) => invoice,
        Err(_) => {
            return Err(Errors::new(&[(
                "setup_invoice",
                "Failed to update xendit invoice payload",
            )]));
        }
    };

    Ok(job_data)
}

async fn message_builder_invoice(
    pool: &PgPool,
    job_data: Value,
    merchant_name: &str,
) -> Result<String, Errors> {
    let invoice_id = match job_data["invoice_id"].as_str() {
        Some(invoice_id) => uuid::Uuid::parse_str(invoice_id).unwrap(),
        None => {
            return Err(Errors::new(&[(
                "message_builder_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let invoice = match Invoice::get_by_id(&pool, &invoice_id).await {
        Ok(invoice) => invoice,
        Err(_) => {
            return Err(Errors::new(&[(
                "message_builder_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let total_amount = match job_data["total_amount"].as_i64() {
        Some(total_amount) => total_amount,
        None => {
            return Err(Errors::new(&[(
                "message_builder_invoice",
                "Failed to prepare invoice",
            )]));
        }
    };

    let customer_name = match job_data["customer_name"].as_str() {
        Some(customer_name) => customer_name,
        None => {
            return Err(Errors::new(&[(
                "message_builder_reminder",
                "Failed to prepare reminder",
            )]));
        }
    };

    let xendit_invoice_payload = invoice.xendit_invoice_payload.unwrap();
    let invoice_url = xendit_invoice_payload["invoice_url"].as_str().unwrap();

    let now = Utc::now();
    let due_time = &now.add(Duration::hours(24));
    let due_time = format!("{}", due_time.format("%d/%m/%Y - %H:%M"));

    let total_amount = format!("Rp{:.2}", total_amount);

    let msg = generate_message();

    let msg = msg.replacen("{}", &customer_name, 1);
    let msg = msg.replacen("{}", &merchant_name, 1);
    let msg = msg.replacen("{}", &total_amount, 1);
    let msg = msg.replacen("{}", &invoice_url, 1);
    let msg = msg.replacen("{}", &due_time, 1);

    Ok(msg)
}

fn message_builder_reminder(job_data: Value, merchant_name: &str) -> Result<String, Errors> {
    let title = match job_data["title"].as_str() {
        Some(title) => title,
        None => {
            return Err(Errors::new(&[(
                "message_builder_reminder",
                "Failed to prepare reminder",
            )]));
        }
    };

    let description = match job_data["description"].as_str() {
        Some(description) => description,
        None => {
            return Err(Errors::new(&[(
                "message_builder_reminder",
                "Failed to prepare reminder",
            )]));
        }
    };

    let customer_name = match job_data["customer_name"].as_str() {
        Some(customer_name) => customer_name,
        None => {
            return Err(Errors::new(&[(
                "message_builder_reminder",
                "Failed to prepare reminder",
            )]));
        }
    };

    let msg = "Hello {}, {} here, we have a message for you \"{}\", \"{}\".".to_string();

    let msg = msg.replacen("{}", &customer_name, 1);
    let msg = msg.replacen("{}", &merchant_name, 1);
    let msg = msg.replacen("{}", &title, 1);
    let msg = msg.replacen("{}", &description, 1);

    Ok(msg)
}
