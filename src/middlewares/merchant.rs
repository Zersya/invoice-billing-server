use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use sqlx::PgPool;

use crate::models::{merchant::Merchant, responses::DefaultResponse};

pub async fn check_merchant<B>(
    State(db): State<PgPool>,
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let paths = req.uri().path().split("/").collect::<Vec<&str>>();

    // If the path is not /merchant/{merchant_id} then skip this middleware
    if paths[1] != "merchant" {
        return next.run(req).await;
    }

    let merchant_id = paths[2];

    let merchant_id = match uuid::Uuid::parse_str(merchant_id) {
        Ok(merchant_id) => merchant_id,
        Err(err) => {
            let body = DefaultResponse::error(
                "Invalid format merchant id",
                err.to_string(),
            )
            .into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    match Merchant::get_by_id(&db, merchant_id).await {
        Ok(merchant) => merchant,
        Err(err) => {
            let body =
                DefaultResponse::error("Merchant not found", err.to_string())
                    .into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    next.run(req).await
}
