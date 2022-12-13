use axum::{
    body::{boxed, Body}, // for `into_make_service`
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

use crate::models::oauth_access_token::{self, OauthAccessToken};

pub async fn check_authentication<B>(
    State(db): State<PgPool>,
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        return Response::builder()
            .status(401)
            .body(boxed(Body::from("Unauthorized")))
            .unwrap();
    }

    let auth_header = auth_header.unwrap().to_str().unwrap();
    let auth_header = auth_header.split(" ").collect::<Vec<&str>>();
    if auth_header.len() != 2 {
        return Response::builder()
            .status(401)
            .body(boxed(Body::from("Unauthorized")))
            .unwrap();
    }

    let access_token = auth_header[1];
    let oauth_access_token = match OauthAccessToken::get_by_access_token(&db, access_token).await {
        Ok(oauth_access_token) => oauth_access_token,
        Err(_) => {
            return Response::builder()
                .status(401)
                .body(boxed(Body::from("Unauthorized")))
                .unwrap();
        }
    };

    if oauth_access_token::is_expired(&oauth_access_token.expires_at) {
        return Response::builder()
            .status(401)
            .body(boxed(Body::from("Unauthorized")))
            .unwrap();
    }

    req.extensions_mut().insert(oauth_access_token.user_id);

    next.run(req).await
}
