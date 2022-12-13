use crate::models::requests::merchant::RequestUpdateMerchant;
use crate::{errors::Errors, models::requests::merchant::RequestCreateMerchant};
use crate::models::merchant::Merchant;
use crate::models::responses::DefaultResponse;
use axum::Extension;
use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_by_authenticated(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Value>, Errors> {
    let merchants = match Merchant::get_by_user_id(&db, user_id).await {
        Ok(merchants) => merchants,
        Err(_) => return Err(Errors::new(&[("user id", "not found")])),
    };

    let body =
        DefaultResponse::ok("get merchants by authenticated user success").with_data(json!(merchants));

    Ok(body.into_response())
}

pub async fn create(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<RequestCreateMerchant>,
) -> Result<Json<Value>, Errors> {
    let merchant = match Merchant::create(&db, &body.name, &body.description, user_id).await {
        Ok(merchant) => merchant,
        Err(_) => return Err(Errors::new(&[("create merchant", "failed")])),
    };

    let body = DefaultResponse::ok("create merchant success").with_data(json!(merchant));

    Ok(body.into_response())
}

pub async fn update(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
    Json(body): Json<RequestUpdateMerchant>,
) -> Result<Json<Value>, Errors> {
    let merchant = match Merchant::update(&db, merchant_id, &body.name, &body.description, user_id).await {
        Ok(merchant) => merchant,
        Err(_) => return Err(Errors::new(&[("update merchant", "failed")])),
    };

    let body = DefaultResponse::ok("update merchant success").with_data(json!(merchant));

    Ok(body.into_response())
}

pub async fn delete(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
) -> Result<Json<Value>, Errors> {
    match Merchant::delete(&db, merchant_id, user_id).await {
        Ok(_) => (),
        Err(_) => return Err(Errors::new(&[("delete merchant", "failed")])),
    };

    let body = DefaultResponse::ok("delete merchant success");

    Ok(body.into_response())
}