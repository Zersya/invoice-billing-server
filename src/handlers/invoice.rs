use crate::models::invoice::Invoice;
use crate::models::requests::invoice::RequestCreateInvoice;
use crate::models::responses::DefaultResponse;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_by_authenticated_user(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let invoices = match Invoice::get_by_merchat_user_id(&db, &user_id).await {
        Ok(invoices) => invoices,
        Err(err) => {
            let body = DefaultResponse::error("get invoices failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get invoices by authenticated user success")
        .with_data(json!(invoices))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn create(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
    Json(body): Json<RequestCreateInvoice>,
) -> Response {

    let tax_rate = 11;
    let tax_amount = body.amount * tax_rate / 100;
    let total_amount = body.amount + tax_amount;

    let invoice = match Invoice::create(
        &db,
        &body.customer_id,
        &merchant_id,
        &body.amount,
        &total_amount,
        &tax_amount,
        &tax_rate,
        &body.invoice_date,
        &user_id,
    )
    .await
    {
        Ok(invoice) => invoice,
        Err(err) => {
            let body = DefaultResponse::error("create invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("create invoice success")
        .with_data(json!(invoice))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}