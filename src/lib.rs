extern crate cron;

use cron::Schedule;
use std::net::SocketAddr;
use std::{str::FromStr};

use axum::{
    routing::{get, post, put},
    Router,
};

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::jobs::spawns::{spawn_job_queue, spawn_set_job_schedule_to_queue};

mod config;
mod errors;
mod handlers;
mod logger;
mod middlewares;
mod models;
mod utils;
mod jobs;
mod repositories;

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
            get(handlers::merchant::get_by_authenticated_user).post(handlers::merchant::create),
        )
        .route(
            "/contact-channels",
            get(handlers::customer::get_contact_channels),
        )
        .route_layer(auth_middleware)
        .route("/login", post(handlers::auth::login))
        .route("/register", post(handlers::auth::register))
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
