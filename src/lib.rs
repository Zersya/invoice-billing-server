use std::net::SocketAddr;

use axum::{
    routing::{delete, get, patch, post, put},
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

pub async fn axum() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(config.pg.as_ref().unwrap().poolmaxsize)
        .connect(config.database_url().as_ref())
        .await
        .expect("Failed to create pool database connection");

    // sqlx::migrate!().run(&pool).await.expect("Failed to migrate the database");

    let middleware = axum::middleware::from_fn_with_state(
        pool.clone(),
        middlewares::authentication::check_authentication,
    );

    let app = Router::with_state(pool)
        .route("/users", get(handlers::user::get_users))
        .route(
            "/merchant/:id",
            patch(handlers::merchant::update).delete(handlers::merchant::delete),
        )
        .route(
            "/merchant",
            get(handlers::merchant::get_by_authenticated).post(handlers::merchant::create),
        )
        .route_layer(middleware)
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
