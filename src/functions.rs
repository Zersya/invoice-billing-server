use lettre::{Message, transport::smtp::authentication::Credentials, SmtpTransport, Transport};

use crate::errors::{DefaultError};


pub async fn send_email_verification(
    name: &String,
    email_recepient: &String,
    url_verification: &String,
) -> Result<(), DefaultError> {
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
        Err(e) => Err(DefaultError {
            value: email_recepient.to_string(),
            message: e.to_string(),
        }),
    }
}
