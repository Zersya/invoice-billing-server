use axum::{response::{Html}, extract::{Query, State}};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use serde::Deserialize;
use sqlx::PgPool;
use validator_derive::Validate;

use crate::{errors::EmailError, models::{verification::Verification, user::User}};

#[derive(Deserialize, Validate, Debug)]
pub struct VerifyQuery {
    pub id: Option<String>,
    pub code: String,
}

pub async fn auth(
    State(db): State<PgPool>,
    Query(query): Query<VerifyQuery>,
) -> Html<&'static str> {

    let now = chrono::Utc::now().naive_utc();

    if query.id.is_some() {
        let id = uuid::Uuid::parse_str(&query.id.unwrap()).unwrap();
        let verification = Verification::get_by_id(&db, id).await.unwrap();

        let expires_at = verification.expires_at;

        if expires_at.is_some() && expires_at < Some(now) {
            return Html("<h1>Verification link has expired</h1>");
        }

        if verification.code == query.code {
            if verification.user_id.is_some() {
                let user = User::get_by_id(&db, verification.user_id.unwrap()).await.unwrap();

                match User::update_verified_at(&db, &user.id, &now).await {
                    Ok(_) => (),
                    Err(e) => {
                        panic!("Error updating user verified_at: {}", e)
                    }
                }
            }
            match Verification::update_status(&db, &verification.id, &"verified".to_string()).await {
                Ok(_) => (),
                Err(e) => {
                    panic!("Error updating verification status: {}", e)
                }
            }

        }

        return Html("<h1>Thank you for verifying your email!</h1>");
    }

    return Html("<h1>Hello world</h1>");
}


pub async fn send_email_verification(
    name: &String,
    email_recepient: &String,
    url_verification: &String,
) -> Result<(), EmailError> {
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
        Err(e) => Err(EmailError {
            email: email_recepient.to_string(),
            message: e.to_string(),
        }),
    }
}
