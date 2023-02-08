use serde_json::json;

use crate::errors::DefaultError;

pub async fn telegram_send_message(chat_id: &i64, message: &str) -> Result<(), DefaultError> {
    let client = reqwest::Client::new();

    let host = std::env::var("TELEGRAM_BASE_URL").unwrap();
    let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap();

    let body = json!({
        "chat_id": chat_id,
        "text": message,
    });

    let result = match client
        .post(format!("{}/bot{}/sendMessage", host, telegram_bot_token))
        .json(&body)
        .send()
        .await
    {
        Ok(_) => (),
        Err(err) => {
            return Err(DefaultError::new(
                err.to_string(),
                "unable to send message telegram".to_string(),
            ))
        }
    };

    Ok(result)
}
