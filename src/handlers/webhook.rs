use crate::models::customer::Customer;
use crate::models::merchant::Merchant;
use crate::models::requests::telegram::TelegramUpdateItem;
use crate::models::responses::DefaultResponse;
use crate::repositories::telegram::telegram_send_message;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use redis::cmd;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::verification::setup_verification;

pub async fn telegram(
    State(db): State<PgPool>,
    Extension(headers): Extension<Vec<(String, String)>>,
    Json(payload): Json<TelegramUpdateItem>,
) -> Response {
    let secret_token = std::env::var("TELEGRAM_SECRET_TOKEN").unwrap();
    let mut is_has_secret_token = false;

    for (key, value) in headers {
        if key == "x-telegram-bot-api-secret-token" {
            is_has_secret_token = true;
            if value != secret_token {
                let body = DefaultResponse::error(
                    "invalid secret token",
                    "invalid secret token".to_string(),
                )
                .into_json();

                return (StatusCode::OK, body).into_response();
            }

            break;
        }
    }

    if !is_has_secret_token {
        let body =
            DefaultResponse::error("no secret token found", "no secret token found".to_string())
                .into_json();

        return (StatusCode::OK, body).into_response();
    }

    let redis_connection = std::env::var("REDIS_CONNECTION").unwrap();
    let client = match redis::Client::open(redis_connection) {
        Ok(client) => client,
        Err(err) => {
            let body =
                DefaultResponse::error("unable to connect to redis", err.to_string()).into_json();

            return (StatusCode::OK, body).into_response();
        }
    };

    let mut con = match client.get_connection() {
        Ok(con) => con,
        Err(err) => {
            let body =
                DefaultResponse::error("unable to connect to redis", err.to_string()).into_json();

            return (StatusCode::OK, body).into_response();
        }
    };

    let key = format!("telegram_{}", payload.message.chat.id);

    if payload.message.text == "/start" {
        telegram_send_message(
            &payload.message.chat.id,
            "Hi, welcome to the telegram bot. Send /connect to connect to the merchant",
        )
        .await
        .unwrap();
    } else if payload.message.text == "/connect" {
        match cmd("SET")
            .arg(key)
            .arg(&payload.message.text)
            .query::<()>(&mut con)
        {
            Ok(_) => Some(()),
            Err(_) => None,
        };

        telegram_send_message(
            &payload.message.chat.id,
            "OK. Send me the merchant code that you get from the merchant",
        )
        .await
        .unwrap();
    } else if payload.message.text == "/clear" {
        match cmd("DEL").arg(&key).query::<Option<()>>(&mut con) {
            Ok(_) => Some(()),
            Err(_) => None,
        };

        match telegram_send_message(
            &payload.message.chat.id,
            "Send /connect to connect to the merchant",
        )
        .await
        {
            Ok(_) => (),
            Err(err) => {
                let body = DefaultResponse::error(&err.message, err.value.to_string()).into_json();

                return (StatusCode::OK, body).into_response();
            }
        }
    } else {
        let current_text = match cmd("GET").arg(&key).query::<Option<String>>(&mut con) {
            Ok(current_text) => current_text,
            Err(_) => None,
        };

        if current_text.is_some() && current_text.as_ref().unwrap() == "/connect" {
            let merchant = match Merchant::get_by_merchant_code(&db, &payload.message.text).await {
                Ok(merchant) => merchant,
                Err(err) => {
                    let msg = "The merchant code is not valid, please check again.";
                    telegram_send_message(&payload.message.chat.id, &msg)
                        .await
                        .unwrap();

                    let body = DefaultResponse::error(&msg, err.to_string()).into_json();

                    return (StatusCode::OK, body).into_response();
                }
            };

            let customer = match Customer::get_by_merchant_id_contact_channel(
                &db,
                &merchant.id,
                &"telegram".to_string(),
                &payload.message.from.username,
            )
            .await
            {
                Ok(result) => result,
                Err(err) => {
                    let msg = "You're not registered in this merchant, please ask admin to register your telegram username.";
                    telegram_send_message(&payload.message.chat.id, &msg)
                        .await
                        .unwrap();

                    let body = DefaultResponse::error(&msg, err.to_string()).into_json();

                    return (StatusCode::OK, body).into_response();
                }
            };

            match setup_verification(
                &db,
                None,
                Some(customer.id),
                customer.contact_channel_name,
                payload.message.chat.id.to_string(),
            )
            .await
            {
                Ok(_) => Some(()),
                Err(err) => {
                    let msg = "Unable to sent verification";
                    telegram_send_message(&payload.message.chat.id, &msg)
                        .await
                        .unwrap();

                    let body = DefaultResponse::error(&msg, err.to_string()).into_json();

                    return (StatusCode::OK, body).into_response();
                }
            };

            match cmd("DEL").arg(&key).query::<Option<()>>(&mut con) {
                Ok(_) => Some(()),
                Err(_) => None,
            };

            let msg = "Thank you for register as customer";
            telegram_send_message(&payload.message.chat.id, msg)
                .await
                .unwrap();
        } else {
            let msg = "Send /connect to connect to the merchant";
            telegram_send_message(&payload.message.chat.id, msg)
                .await
                .unwrap();
        }
    }

    let body = DefaultResponse::ok("success webhook telegram").into_json();

    (StatusCode::OK, body).into_response()
}
