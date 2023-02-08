use axum::http::HeaderValue;

use crate::errors::DefaultError;

pub async fn whatsapp_send_message(phone_number: &str, message: &str) -> Result<(), DefaultError> {
    let client = reqwest::Client::new();

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
        Err(e) => return Err(DefaultError {
            value: phone_number.to_string(),
            message: e.to_string(),
        }),
    };

    Ok(())
}
