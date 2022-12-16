use serde_json::{json, Value};

use crate::errors::Errors;

pub async fn send_invoice_to_xendit(
    external_id: &str,
    amount: &i32,
    description: &str,
) -> Result<Value, Errors> {
    let client = reqwest::Client::new();

    let host = std::env::var("XENDIT_BASE_URL").unwrap();
    let xendit_secret_key = std::env::var("XENDIT_SECRET_KEY").unwrap();
    // let xendit_public_key = std::env::var("XENDIT_PUBLIC_KEY").unwrap();

    let body = json!({
        "external_id": external_id,
        "amount": amount,
        "description": description
    });

    let response = match client
        .post(format!("{}/v2/invoices", host))
        .basic_auth(xendit_secret_key, Some(""))
        .json(&body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => {
            return Err(Errors::new(&[(
                "whatsapp_send_message",
                "Failed to send message",
            )]));
        }
    };

    let json = match response.json().await {
        Ok(json) => json,
        Err(_) => {
            return Err(Errors::new(&[(
                "whatsapp_send_message",
                "Failed to receive body response",
            )]));
        }
    };

    return Ok(json);
}
