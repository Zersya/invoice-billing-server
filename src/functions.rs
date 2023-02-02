use axum::http::HeaderValue;
use lettre::{Message, transport::smtp::authentication::Credentials, SmtpTransport, Transport};

use crate::errors::{ChannelError};

pub async fn whatsapp_send_message(phone_number: &str, message: &str) -> Result<(), ChannelError> {
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
        Err(e) => return Err(ChannelError {
            value: phone_number.to_string(),
            message: e.to_string(),
        }),
        // Err(_) => {
        //     return Err(Errors::new(&[(
        //         "whatsapp_send_message",
        //         "Failed to send message",
        //     )]));
        // }
    };

    Ok(())
}


pub async fn send_email_verification(
    name: &String,
    email_recepient: &String,
    url_verification: &String,
) -> Result<(), ChannelError> {
    let message = format!(
        "Hello {}, thank you for registering in Inving. Please click this link to verify your account: \n\n{}",
        name, url_verification, 
    );

    let email = Message::builder()
        .from("Verification <hello@inving.co>".parse().unwrap())
        .to(email_recepient.parse().unwrap())
        .subject("Inving - Email Verification")
        .body(message.clone())
        .unwrap();

    let password = std::env::var("EMAIL_SENDGRID_API_KEY").unwrap();
    let creds = Credentials::new("apikey".to_string(), password);

    let mailer = SmtpTransport::relay("smtp.sendgrid.net")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(ChannelError {
            value: email_recepient.to_string(),
            message: e.to_string(),
        }),
    }
}
