use axum::{
    extract::{Query, State},
    response::Html,
};
use serde::Deserialize;
use sqlx::PgPool;
use validator_derive::Validate;

use crate::{
    errors::ChannelError,
    functions::{send_email_verification, whatsapp_send_message},
    models::{customer::Customer, user::User, verification::Verification},
};

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

        if verification.status == "verified" {
            return Html("<h1>Verification link has already been used</h1>");
        }

        if verification.code == query.code {
            if verification.user_id.is_some() {
                let user = User::get_by_id(&db, verification.user_id.unwrap())
                    .await
                    .unwrap();

                match User::update_verified_at(&db, &user.id, &now).await {
                    Ok(_) => (),
                    Err(e) => {
                        panic!("Error updating user verified_at: {}", e)
                    }
                }
            }

            if verification.customer_id.is_some() {
                let customer = Customer::get_by_id_only(&db, verification.customer_id.unwrap())
                    .await
                    .unwrap();

                match Customer::update_verified_at(&db, &customer.id, &now).await {
                    Ok(_) => (),
                    Err(e) => {
                        panic!("Error updating customer verified_at: {}", e)
                    }
                }
            }

            match Verification::update_status(&db, &verification.id, &"verified".to_string()).await
            {
                Ok(_) => (),
                Err(e) => {
                    panic!("Error updating verification status: {}", e)
                }
            }
        }

        return Html("<h1>Thank you for verifying!</h1>");
    }

    return Html("<h1>Hello world</h1>");
}

pub async fn setup_verification(
    db: &sqlx::PgPool,
    user_id: Option<uuid::Uuid>,
    customer_id: Option<uuid::Uuid>,
    contact_channel_name: String,
    contact_value: String,
) -> Result<(), ChannelError> {
    let code = rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>();

    let verification = match Verification::create(&db, user_id, customer_id, &code).await {
        Ok(verification) => verification,
        Err(err) => {
            return Err(ChannelError {
                value: "setup verification error".to_string(),
                message: err.to_string(),
            });
        }
    };

    let base_url = std::env::var("APP_HOST").unwrap();
    let url_verification = format!(
        "http://{}/verify?code={}&id={}",
        base_url, code, verification.id
    );

    let recepient_name = if user_id.is_some() {
        let user = match User::get_by_id(&db, user_id.unwrap()).await {
            Ok(user) => user,
            Err(err) => {
                return Err(ChannelError {
                    value: "setup verification error".to_string(),
                    message: err.to_string(),
                });
            }
        };

        user.name
    } else {
        let customer = match Customer::get_by_id_only(&db, customer_id.unwrap()).await {
            Ok(customer) => customer,
            Err(err) => {
                return Err(ChannelError {
                    value: "setup verification error".to_string(),
                    message: err.to_string(),
                });
            }
        };

        customer.name
    };

    if contact_channel_name == "email" {
        match send_email_verification(&recepient_name, &contact_value, &url_verification).await {
            Ok(_) => (),
            Err(err) => {
                return Err(ChannelError {
                    value: "setup verification error".to_string(),
                    message: err.to_string(),
                });
            }
        };
    } else if contact_channel_name == "whatsapp" {
        let message = format!("Hi {}, please verify your account by clicking this link: {}", recepient_name, url_verification);
        match whatsapp_send_message(contact_value.as_str(), message.as_str()).await {
            Ok(_) => (),
            Err(err) => {
                return Err(ChannelError{
                    value: "setup verification error".to_string(),
                    message: err.to_string(),
                });
            }
        }
    }

    Ok(())
}
