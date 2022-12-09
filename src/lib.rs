use std::net::SocketAddr;

use axum::{routing::{get, post, put, patch, delete}, Router};

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

mod config;
mod handlers;
mod models;
mod errors;
mod logger;

pub async fn axum() {
    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(config.pg.as_ref().unwrap().poolmaxsize)
        .connect(config.database_url().as_ref())
        .await
        .expect("Failed to create pool database connection");

    // sqlx::migrate!().run(&pool).await.expect("Failed to migrate the database");

    let app = Router::with_state(pool)
        .route("/", get(handlers::user::hello_world))
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/users", get(handlers::user::user_list));
        
    let host = &config.server.as_ref().unwrap().host;
    let port = &config.server.as_ref().unwrap().port;
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
