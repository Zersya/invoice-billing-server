use crate::models::user::User;

use axum::{response::{Json, IntoResponse}, extract::State};
use sqlx::PgPool;

use axum::response::Html;

pub async fn hello_world() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

pub async fn user_list(State(db): State<PgPool>,) -> impl IntoResponse {
    
    let users = User::get_all(db).await.unwrap();

    Json(users)
}
