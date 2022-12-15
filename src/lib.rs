extern crate cron;

use axum::http::HeaderValue;
use chrono::Utc;
use cron::Schedule;
use errors::Errors;
use models::customer_contact_channel::CustomerContactChannel;
use models::job_queue::JobQueue;
use models::job_schedule::JobSchedule;
use reqwest::Error;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::time::Duration;
use std::{str::FromStr};
use tokio::time::interval;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod errors;
mod handlers;
mod logger;
mod middlewares;
mod models;
mod utils;

pub async fn axum() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = PgPoolOptions::new()
        .min_connections(config.pg.as_ref().unwrap().poolminsize)
        .max_connections(config.pg.as_ref().unwrap().poolmaxsize)
        .connect(config.database_url().as_ref())
        .await
        .expect("Failed to create pool database connection");

    let expression = "*/30 * * * * * *";
    let schedule = Schedule::from_str(expression).unwrap();

    spawn_job_queue(pool.clone(), schedule).await;

    spawn_set_job_schedule_to_queue(pool.clone()).await;

    let auth_middleware = axum::middleware::from_fn_with_state(
        pool.clone(),
        middlewares::authentication::check_authentication,
    );

    let merchant_middleware =
        axum::middleware::from_fn_with_state(pool.clone(), middlewares::merchant::check_merchant);

    let app = Router::with_state(pool.clone())
        .route("/users", get(handlers::user::get_users))
        .route(
            "/merchant/:id/invoice/:id/set-schedule",
            put(handlers::invoice::set_invoice_scheduler),
        )
        .route(
            "/merchant/:id/invoice",
            get(handlers::invoice::get_by_authenticated_user).post(handlers::invoice::create),
        )
        .route(
            "/merchant/:id/customer/:id",
            put(handlers::customer::update).delete(handlers::customer::delete),
        )
        .route(
            "/merchant/:id/customer/all",
            get(handlers::customer::get_by_authenticated_user),
        )
        .route(
            "/merchant/:id/customer",
            get(handlers::customer::get_by_merchant_id).post(handlers::customer::create),
        )
        .route(
            "/merchant/:id",
            put(handlers::merchant::update).delete(handlers::merchant::delete),
        )
        .route_layer(merchant_middleware)
        .route(
            "/merchant",
            get(handlers::merchant::get_by_authenticated_user), // .post(handlers::merchant::create),
        )
        .route(
            "/contact-channels",
            get(handlers::customer::get_contact_channels),
        )
        .route_layer(auth_middleware)
        .route("/login", post(handlers::auth::login))
        // .route("/register", post(handlers::auth::register))
        .route("/", get(handlers::user::hello_world));

    let host = &config.server.as_ref().unwrap().host;
    let port = &config.server.as_ref().unwrap().port;
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();

    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn spawn_job_queue(pool: PgPool, schedule: Schedule) {
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
                JobSchedule::update_status(&pool, job.job_schedule_id.unwrap(), "in_progress")
                    .await.expect("Failed to update job schedule status to in_progress");
            }

            JobQueue::update_status(&pool, &job.id, "in_progress")
                .await
                .expect("Failed to update job queue status to in_progress");


            if job.job_data.is_none() {
                JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                continue;
            }

            let job_data = match serde_json::from_value::<serde_json::Value>(job.job_data.unwrap())
            {
                Ok(job_data) => job_data,
                Err(_) => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");

                    continue;
                }
            };

            let customer_id = match job_data["customer_id"].as_str() {
                Some(phone_number) => uuid::Uuid::parse_str(phone_number).unwrap(),
                None => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                    continue;
                }
            };

            let merchant_id = match job_data["merchant_id"].as_str() {
                Some(merchant_id) => uuid::Uuid::parse_str(merchant_id).unwrap(),
                None => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                    continue;
                }
            };

            let amount = match job_data["amount"].as_i64() {
                Some(amount) => amount.to_string(),
                None => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                    continue;
                }
            };

            let customer_contact_channels =
                match CustomerContactChannel::get_customer_contact_channels_by_customer_and_merchant(
                    &pool,
                    &customer_id,
                    &merchant_id,
                ).await {
                    Ok(customer_contact_channels) => customer_contact_channels,
                    Err(_) => {
                        JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                        continue;
                    }
                };

            // This code finds the whatsapp contact channel, if it exists.
            let whatsapp_contact_channel = match customer_contact_channels
                .iter()
                .find(|contact_channel| contact_channel.name == "whatsapp")
            {
                Some(whatsapp_contact_channel) => whatsapp_contact_channel,
                None => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                    continue;
                }
            };

            match whatsapp_send_message(whatsapp_contact_channel.value.as_str(), format!("The total amount due is {}. Please remit payment within 30 days to avoid late fees.", amount).as_str(), &schedule).await {
                Ok(_) => {
                    JobQueue::update_status(&pool, &job.id, "completed")
                        .await
                        .expect("Failed to update job queue status to completed");
                
                    ()
                },
                Err(_) => {
                    JobQueue::update_status(&pool, &job.id, "failed")
                        .await
                        .expect("Failed to update job queue status to failed");
                    
                        continue;
                }
            }

            if job.job_schedule_id.is_some() {
                JobSchedule::update_status(&pool, job.job_schedule_id.unwrap(), "completed")
                    .await
                    .expect("Failed to update job schedule status to completed");
            }
        }
    });
}

async fn spawn_set_job_schedule_to_queue(pool: PgPool) {
    tokio::spawn(async move {
        // Use an interval to perform the check at regular intervals.
        let mut interval = interval(Duration::from_secs(15));

        loop {
            interval.tick().await;
            set_job_schedule_to_queue(pool.clone()).await;
        }
    });
}

async fn whatsapp_send_message(
    phone_number: &str,
    message: &str,
    schedule: &Schedule,
) -> Result<(), Errors> {
    let client = reqwest::Client::new();

    let now = Utc::now();

    let schedule = schedule.upcoming(Utc).take(1);
    let datetime = schedule.last().unwrap();

    println!("Datetime {}, Now {}", datetime, now);


    // when different is less that 500 ms, then run the task
    // this is to avoid task running before the time sleeps changes
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

async fn set_job_schedule_to_queue(pool: PgPool) {
    // let mut conn = pool.acquire().await.unwrap();

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
