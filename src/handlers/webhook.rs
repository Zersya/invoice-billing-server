use crate::models::requests::telegram::TelegramUpdateItem;
use crate::models::responses::DefaultResponse;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

pub async fn telegram(
    State(db): State<PgPool>,
    Extension(headers): Extension<Vec<(String, String)>>,
    Json(payload): Json<TelegramUpdateItem>,
) -> Response {
    let secret_token = std::env::var("TELEGRAM_SECRET_TOKEN").unwrap();

    for (key, value) in headers {
        if key == "x-telegram-bot-api-secret-token" {
            if value != secret_token {
                let body = DefaultResponse::ok("invalid secret token").into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }

            break;
        }
    }

    let client = reqwest::Client::new();

    let host = std::env::var("TELEGRAM_BASE_URL").unwrap();
    let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap();

    if payload.message.text == "/start" {
        let body = json!({
            "chat_id": payload.message.chat.id,
            "text": "Hi, welcome to the telegram bot. Send /connect to connect to the merchant",
        });

        match client
            .post(format!("{}/bot{}/sendMessage", host, telegram_bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => {
                let body = DefaultResponse::ok("unable to send telegram request").into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };
    } else if payload.message.text == "/connect" {
        let body = json!({
            "chat_id": payload.message.chat.id,
            "text": "OK. Send me the merchant code that you get from the merchant",
        });

        match client
            .post(format!("{}/bot{}/sendMessage", host, telegram_bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => {
                let body = DefaultResponse::ok("unable to send telegram request").into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };
    } else {
        let body = json!({
            "chat_id": payload.message.chat.id,
            "text": "Send /connect to connect to the merchant",
        });

        match client
            .post(format!("{}/bot{}/sendMessage", host, telegram_bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => {
                let body = DefaultResponse::ok("unable to send telegram request").into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };
    }

    let body = DefaultResponse::ok("success webhook telegram").into_json();

    (StatusCode::OK, body).into_response()
}
