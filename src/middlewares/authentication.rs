use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use sqlx::PgPool;

use crate::models::{
    oauth_access_token::{self, OauthAccessToken},
    responses::DefaultResponse,
};

pub async fn check_authentication<B>(
    State(db): State<PgPool>,
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        let body = DefaultResponse::unauthorized("Unauthorized", Some("No authorization found".to_string()))
            .into_json();

        return (StatusCode::UNAUTHORIZED, body).into_response();
    }

    let auth_header = auth_header.unwrap().to_str().unwrap();
    let auth_header = auth_header.split(" ").collect::<Vec<&str>>();
    if auth_header.len() != 2 {
        let body =
            DefaultResponse::unauthorized("Unauthorized", Some("Invalid format authorization".to_string()))
                .into_json();

        return (StatusCode::UNAUTHORIZED, body).into_response();
    }

    let access_token = auth_header[1];
    let oauth_access_token = match OauthAccessToken::get_by_access_token(&db, access_token).await {
        Ok(oauth_access_token) => oauth_access_token,
        Err(err) => {
            let body =
                DefaultResponse::unauthorized("Unauthorized", Some(err.to_string()))
                    .into_json();

            return (StatusCode::UNAUTHORIZED, body).into_response();
        }
    };

    if oauth_access_token::is_expired(&oauth_access_token.expires_at) {
        let body = DefaultResponse::unauthorized("Unauthorized", Some("Token expired".to_string())).into_json();

        return (StatusCode::UNAUTHORIZED, body).into_response();
    }

    req.extensions_mut().insert(oauth_access_token.user_id);

    next.run(req).await
}
