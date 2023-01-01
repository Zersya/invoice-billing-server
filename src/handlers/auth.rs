use crate::errors::{Errors, FieldValidator};
use crate::models::oauth_access_token::OauthAccessToken;
use crate::models::requests::auth::{RequestLogin, RequestRegister};
use crate::models::responses::DefaultResponse;
use crate::models::tester::Tester;
use crate::models::user::User;

use argon2::{self, Config};
use argon2::{ThreadMode, Variant, Version};

use axum::response::{IntoResponse, Response};
use axum::{extract::State, response::Json};
use crypto_hash::{hex_digest, Algorithm};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

pub async fn register(State(db): State<PgPool>, Json(payload): Json<RequestRegister>) -> Response {
    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    let email = extractor.extract("email", Some(payload.email));
    let password = extractor.extract("password", Some(payload.password));
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response(),
    }

    match User::get_by_email(&db, &email).await {
        Ok(err) => {
            let body = DefaultResponse::error("Email already exist", err.email).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
        Err(_) => (),
    }

    let salt = std::env::var("APPKEY").unwrap();

    let config = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 512,
        time_cost: 2,
        lanes: 1,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };

    let email = email.trim().to_string().to_lowercase();
    let password = password.trim().to_string().to_lowercase();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt.as_bytes(), &config).unwrap();

    let user = match User::create(&db, &name, &email, &hash).await {
        Ok(user) => user,
        Err(err) => {
            let body = DefaultResponse::error("register failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("register success, we will inform you when your account is ready")
        .with_data(json!(user))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn login(State(db): State<PgPool>, Json(payload): Json<RequestLogin>) -> Response {
    let mut extractor = FieldValidator::validate(&payload);

    let email = extractor.extract("email", Some(payload.email));
    let password = extractor.extract("password", Some(payload.password));
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response(),
    }

    let user = match User::get_by_email(&db, &email).await {
        Ok(user) => user,
        Err(err) => {
            let body = DefaultResponse::error("login failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let tester = match Tester::get_by_user_id(&db, user.id).await {
        Ok(tester) => tester,
        Err(err) => {
            let body = DefaultResponse::error("It looks like you have not registered as a tester.", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    // if user.status == String::from("inactive") {
    //     let body =
    //         DefaultResponse::error("login failed", String::from("user is inactive")).into_json();

    //     return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    // }

    let salt = std::env::var("APPKEY").unwrap();
    let config = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 512,
        time_cost: 2,
        lanes: 1,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };

    let email = email.trim().to_string().to_lowercase();
    let password = password.trim().to_string().to_lowercase();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt.as_bytes(), &config).unwrap();

    let matches = argon2::verify_encoded(&hash, password.as_bytes()).unwrap();
    assert!(matches);

    let user = match User::login(&db, email, hash).await {
        Ok(user) => user,
        Err(err) => {
            let body = DefaultResponse::error("login failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let token = match set_access_token(&db, &user, &salt).await {
        Ok(token) => token,
        Err(err) => return (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response(),
    };

    let body = DefaultResponse::ok("login success")
        .with_access_token(token.access_token)
        .with_data(json!(user))
        .into_json();

    (StatusCode::OK, body).into_response()
}

async fn set_access_token(
    db: &PgPool,
    user: &User,
    salt: &String,
) -> Result<OauthAccessToken, Errors> {
    let timestamp = chrono::Utc::now().timestamp_millis().to_string();

    let payload = format!("{}/{}/{}/", &user.id, &salt, timestamp);

    let max_access_token_count = 2; // Only allow 2 access tokens per user
    let access_token = hex_digest(Algorithm::SHA256, payload.as_bytes());

    let count_access_token = OauthAccessToken::get_count_by_user_id(&db, user.id)
        .await
        .unwrap();

    if count_access_token >= max_access_token_count {
        let token = OauthAccessToken::get_by_user_id(&db, user.id)
            .await
            .expect("failed to get tokens");
        OauthAccessToken::delete(&db, token.access_token)
            .await
            .expect("failed to delete token");
    }

    match OauthAccessToken::create(&db, access_token, user.id).await {
        Ok(token) => Ok(token),
        Err(_) => return Err(Errors::new(&[("token generation", "failed")])),
    }
}
