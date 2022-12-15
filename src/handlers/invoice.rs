use crate::models::invoice::Invoice;
use crate::models::job_schedule::JobSchedule;
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

    let body = DefaultResponse::created("create invoice success")
        .with_data(json!(invoice))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn set_invoice_scheduler(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((_, invoice_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let invoice = match Invoice::get_by_id(&db, &invoice_id).await {
        Ok(invoice) => invoice,
        Err(err) => {
            let body = DefaultResponse::error("get invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let job_schedule = match JobSchedule::create(
        &db,
        "send_invoice",
        Some(json!({
            "invoice_id": invoice.id,
            "customer_id": invoice.customer_id,
            "merchant_id": invoice.merchant_id,
            "amount": invoice.amount,
            "total_amount": invoice.total_amount,
            "tax_amount": invoice.tax_amount,
            "tax_rate": invoice.tax_rate,
            "invoice_date": invoice.invoice_date,
            "created_by": user_id,
        })),
        &invoice.invoice_date,
        None,
        None,
        None,
        "scheduled",
        None,
        None,
    )
    .await
    {
        Ok(job_schedule) => job_schedule,
        Err(err) => {
            let body =
                DefaultResponse::error("create job schedule failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("set invoice scheduler success")
        .with_data(json!(job_schedule))
        .into_json();

    (StatusCode::OK, body).into_response()
}
