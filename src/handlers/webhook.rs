use crate::models::requests::telegram::TelegramUpdateItem;
use crate::models::responses::DefaultResponse;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use redis::cmd;
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

    let redis_connection = std::env::var("REDIS_CONNECTION").unwrap();
    let client = match redis::Client::open(redis_connection) {
        Ok(client) => client,
        Err(_) => {
            let body = DefaultResponse::ok("unable to connect to redis").into_json();

            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    let mut con = match client.get_connection() {
        Ok(con) => con,
        Err(err) => {
            let body =
                DefaultResponse::error("unable to connect to redis", err.to_string()).into_json();

            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    let client = reqwest::Client::new();

    let host = std::env::var("TELEGRAM_BASE_URL").unwrap();
    let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap();

    let key = format!("telegram_{}", payload.message.chat.id);

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
            Err(err) => {
                let body =
                    DefaultResponse::error("unable to send telegram request", err.to_string())
                        .into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };
    } else if payload.message.text == "/connect" {
        let body = json!({
            "chat_id": payload.message.chat.id,
            "text": "OK. Send me the merchant code that you get from the merchant",
        });

        match cmd("SET")
            .arg(key)
            .arg(&payload.message.text)
            .query::<()>(&mut con)
        {
            Ok(result) => result,
            Err(err) => {
                let body = DefaultResponse::error("unable to set value to redis", err.to_string())
                    .into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };

        match client
            .post(format!("{}/bot{}/sendMessage", host, telegram_bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => {
                let body =
                    DefaultResponse::error("unable to send telegram request", err.to_string())
                        .into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };
    } else {
        let current_text = match cmd("GET").arg(&key).query::<Option<String>>(&mut con) {
            Ok(current_text) => current_text,
            Err(err) => {
                let body = DefaultResponse::error("unable to get redis value", err.to_string())
                    .into_json();

                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };

        if current_text.is_some() && current_text.unwrap() == "/connect" {
            match cmd("DEL").arg(&key).query::<()>(&mut con) {
                Ok(result) => result,
                Err(err) => {
                    let body =
                        DefaultResponse::error("unable to set value to redis", err.to_string())
                            .into_json();

                    return (StatusCode::BAD_REQUEST, body).into_response();
                }
            };

            let body = json!({
                "chat_id": payload.message.chat.id,
                "text": "Thank you for register as customer",
            });

            match client
                .post(format!("{}/bot{}/sendMessage", host, telegram_bot_token))
                .json(&body)
                .send()
                .await
            {
                Ok(res) => res,
                Err(err) => {
                    let body =
                        DefaultResponse::error("unable to send telegram request", err.to_string())
                            .into_json();

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
                Err(err) => {
                    let body =
                        DefaultResponse::error("unable to send telegram request", err.to_string())
                            .into_json();

                    return (StatusCode::BAD_REQUEST, body).into_response();
                }
            };
        }
    }

    let body = DefaultResponse::ok("success webhook telegram").into_json();

    (StatusCode::OK, body).into_response()
}
