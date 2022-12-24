use crate::models::merchant::Merchant;
use crate::models::requests::merchant::RequestUpdateMerchant;
use crate::models::responses::DefaultResponse;
use crate::{models::requests::merchant::RequestCreateMerchant};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::{json};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_by_authenticated_user(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let merchants = match Merchant::get_by_user_id(&db, &user_id).await {
        Ok(merchants) => merchants,
        Err(err) => {
            let body = DefaultResponse::error("get merchants failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get merchants by authenticated user success")
        .with_data(json!(merchants))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn create(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<RequestCreateMerchant>,
) -> Response {
    if body.name.is_empty() {
        let body = DefaultResponse::error("name is required", "name is empty".to_string())
            .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if body.description.is_empty() {
        let body = DefaultResponse::error("description is required", "description is empty".to_string())
            .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }


    let merchant = match Merchant::create(&db, &body.name, &body.description, &user_id).await {
        Ok(merchant) => merchant,
        Err(err) => {
            let body =
                DefaultResponse::error("create merchant failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("create merchant success")
        .with_data(json!(merchant))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn update(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
    Json(body): Json<RequestUpdateMerchant>,
) -> Response {
    let merchant =
        match Merchant::update(&db, merchant_id, &body.name, &body.description, &user_id).await {
            Ok(merchant) => merchant,
            Err(err) => {
                let body =
                    DefaultResponse::error("update merchant failed", err.to_string()).into_json();

                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        };

    let body = DefaultResponse::ok("update merchant success")
        .with_data(json!(merchant))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn delete(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
) -> Response {
    match Merchant::delete(&db, merchant_id, &user_id).await {
        Ok(_) => (),
        Err(err) => {
            let body =
                DefaultResponse::error("delete merchant failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("delete merchant success").into_json();

    (StatusCode::OK, body).into_response()
}
