use crate::models::requests::merchant::RequestUpdateMerchant;
use crate::{errors::Errors, models::requests::merchant::RequestCreateMerchant};
use crate::models::merchant::Merchant;
use crate::models::responses::DefaultResponse;
use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_by_user_id(
    State(db): State<PgPool>,
    user_id: uuid::Uuid,
) -> Result<Json<Value>, Errors> {
    let merchants = match Merchant::get_by_user_id(&db, user_id).await {
        Ok(merchants) => merchants,
        Err(_) => return Err(Errors::new(&[("merchant", "not found")])),
    };

    let body =
        DefaultResponse::ok("get merchant by user id successfully").with_data(json!(merchants));

    Ok(body.into_response())
}

pub async fn create(
    State(db): State<PgPool>,
    Json(body): Json<RequestCreateMerchant>,
) -> Result<Json<Value>, Errors> {
    let merchant = match Merchant::create(&db, &body.name, &body.description, body.user_id).await {
        Ok(merchant) => merchant,
        Err(_) => return Err(Errors::new(&[("create merchant", "failed")])),
    };

    let body = DefaultResponse::ok("create merchant successfully").with_data(json!(merchant));

    Ok(body.into_response())
}

pub async fn update(
    State(db): State<PgPool>,
    Path((merchant_id,)): Path<(Uuid,)>,
    Json(body): Json<RequestUpdateMerchant>,
) -> Result<Json<Value>, Errors> {
    let merchant = match Merchant::update(&db, merchant_id, &body.name, &body.description).await {
        Ok(merchant) => merchant,
        Err(_) => return Err(Errors::new(&[("update merchant", "failed")])),
    };

    let body = DefaultResponse::ok("update merchant successfully").with_data(json!(merchant));

    Ok(body.into_response())
}
