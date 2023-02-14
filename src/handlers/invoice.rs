use std::ops::Add;

use crate::models::customer::Customer;
use crate::models::invoice::Invoice;
use crate::models::item::Item;
use crate::models::job_queue::JobQueue;
use crate::models::job_schedule::JobSchedule;
use crate::models::merchant::Merchant;
use crate::models::requests::invoice::{RequestAddInvoiceItem, RequestCreateInvoice};
use crate::models::requests::invoice_schedule::{
    RequestInvoiceSchedule, RequestSetStatusInvoiceSchedule,
};
use crate::models::responses::DefaultResponse;
use crate::repositories::invoice::send_invoice_to_xendit;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use rust_decimal::prelude::ToPrimitive;
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

pub async fn get_by_merchant_id(
    State(db): State<PgPool>,
    Path((merchant_id,)): Path<(Uuid,)>,
) -> Response {
    let invoices = match Invoice::get_by_merchant_id(&db, &merchant_id).await {
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

    let now = chrono::Utc::now().naive_utc();

    let invoice_number =
        "INVC-".to_owned() + &user_id.to_string() + "-" + now.timestamp().to_string().as_str();

    let result = match send_invoice_to_xendit(
        &invoice_number,
        &total_amount,
        &body.to_string_custom_amount(total_amount),
    )
    .await
    {
        Ok(payload) => payload,
        Err(_) => {
            let body = DefaultResponse::error(
                "Failed to send invoice, please try again later",
                "send invoice to xendit failed".to_string(),
            )
            .into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    if body.title.is_none() {
        let body = DefaultResponse::error(
            "Failed to create invoice, please provide title",
            "title is required".to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let invoice = match Invoice::create(
        &db,
        &invoice_number,
        &body.customer_id,
        &merchant_id,
        &body.amount,
        &total_amount,
        &tax_amount,
        &tax_rate,
        &body.invoice_date.expect("invoice date is required"),
        &user_id,
        body.title.as_deref(),
        body.description.as_deref(),
    )
    .await
    {
        Ok(invoice) => invoice,
        Err(err) => {
            let body = DefaultResponse::error("create invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let invoice = match Invoice::update_xendit_invoice_payload(&db, &invoice.id, &result).await {
        Ok(invoice) => invoice,
        Err(err) => {
            let body = DefaultResponse::error("update invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::created("create invoice success")
        .with_data(json!(invoice))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn set_invoice_status(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((_, invoice_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RequestSetStatusInvoiceSchedule>,
) -> Response {
    match validator::Validate::validate(&body) {
        Ok(_) => (),
        Err(err) => {
            let body =
                DefaultResponse::error(err.to_string().as_str(), err.to_string()).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    }

    match JobSchedule::update_status_by_invoice_id(
        &db,
        &body.status,
        invoice_id.to_string().as_str(),
        user_id.to_string().as_str(),
    )
    .await
    {
        Ok(_) => (),
        Err(err) => {
            let body = DefaultResponse::error(
                "no job schedule with related invoice found",
                err.to_string(),
            )
            .into_json();

            return (StatusCode::NOT_FOUND, body).into_response();
        }
    };

    match JobQueue::update_status_by_invoice_id(
        &db,
        &body.status,
        invoice_id.to_string().as_str(),
        user_id.to_string().as_str(),
    )
    .await
    {
        Ok(_) => (),
        Err(_) => {
            let body =
                DefaultResponse::ok("update job status success with running job has been canceled")
                    .into_json();

            return (StatusCode::OK, body).into_response();
        }
    }

    let body = DefaultResponse::ok("update job status success").into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn set_invoice_scheduler(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((_, invoice_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RequestInvoiceSchedule>,
) -> Response {
    match validator::Validate::validate(&body) {
        Ok(_) => (),
        Err(err) => {
            let body =
                DefaultResponse::error(err.to_string().as_str(), err.to_string()).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    }

    if body.is_recurring && (body.start_at.is_none() || body.end_at.is_none()) {
        let body = DefaultResponse::error(
            "start_at and end_at is required for recurring invoice",
            invoice_id.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    match JobSchedule::get_by_job_data_json_by_invoice_id(&db, invoice_id.to_string().as_str())
        .await
    {
        Ok(_) => {
            let body = DefaultResponse::error(
                "Invoice is scheduled",
                "Found job schedule for invoice".to_string(),
            )
            .into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
        Err(_) => (),
    };

    let now = chrono::Utc::now();

    let start_at = if !body.is_recurring {
        now.add(chrono::Duration::seconds(5)).naive_utc()
    } else {
        body.start_at.unwrap()
    };

    let end_at = if !body.is_recurring {
        now.add(chrono::Duration::seconds(10)).naive_utc()
    } else {
        body.end_at.unwrap()
    };

    if end_at < start_at {
        let body = DefaultResponse::error(
            "end_at must be greater than start_at",
            invoice_id.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if start_at < now.naive_utc() {
        let body = DefaultResponse::error(
            format!("start_at must be greater than current time ( {} )", now).as_str(),
            invoice_id.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if body.is_recurring && end_at - start_at < chrono::Duration::days(5) {
        let body =
            DefaultResponse::error("duration must be more than 5 days", invoice_id.to_string())
                .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if body.is_recurring && body.repeat_interval_type.is_none() {
        let body = DefaultResponse::error(
            "repeat_interval_type is required for recurring invoice",
            invoice_id.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let repeat_interval_type = match body.repeat_interval_type {
        Some(repeat_interval_type) => repeat_interval_type,
        None => "ONCE".to_string(),
    };

    let repeat_interval = if repeat_interval_type == "ONCE" {
        let duration = 5;
        duration
    } else if repeat_interval_type == "PERMINUTE" {
        let duration = chrono::Duration::minutes(1).num_seconds();
        duration
    } else if repeat_interval_type == "HOURLY" {
        let duration = chrono::Duration::hours(1).num_seconds();
        duration
    } else if repeat_interval_type == "DAILY" {
        let duration = chrono::Duration::days(1).num_seconds();
        duration
    } else if repeat_interval_type == "WEEKLY" {
        let duration = chrono::Duration::weeks(1).num_seconds();
        duration
    } else if repeat_interval_type == "MONTHLY" {
        let duration = chrono::Duration::weeks(4).num_seconds();
        duration
    } else {
        let duration = chrono::Duration::weeks(1).num_seconds();
        duration
    };

    // repeat count based on start and end date and repeat interval
    let repeat_count = if !body.is_recurring {
        0
    } else {
        (end_at - start_at).num_seconds() / repeat_interval
    };

    let invoice = match Invoice::get_by_id(&db, &invoice_id).await {
        Ok(invoice) => invoice,
        Err(err) => {
            let body = DefaultResponse::error("get invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let customer = match Customer::get_by_id(&db, invoice.customer_id, &invoice.merchant_id).await {
        Ok(customer) => customer,
        Err(err) => {
            let body = DefaultResponse::error("get customer failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let merchant = match Merchant::get_by_id(&db, invoice.merchant_id).await {
        Ok(merchant) => merchant,
        Err(err) => {
            let body = DefaultResponse::error("get merchant failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let job_schedule = match JobSchedule::create(
        &db,
        "send_invoice",
        Some(json!({
            "invoice_id": invoice.id,
            "invoice_number": invoice.invoice_number,
            "title": invoice.title,
            "description": invoice.description,
            "customer_id": invoice.customer_id,
            "customer_name": customer.name,
            "merchant_id": invoice.merchant_id,
            "merchant_name": merchant.name,
            "amount": invoice.amount,
            "total_amount": invoice.total_amount,
            "tax_amount": invoice.tax_amount,
            "tax_rate": invoice.tax_rate,
            "invoice_date": invoice.invoice_date,
            "created_by": user_id,
        })),
        &start_at,
        Some(repeat_interval),
        repeat_count.to_i32(),
        repeat_count.to_i32(),
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

pub async fn add_item_to_invoice(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((_, invoice_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RequestAddInvoiceItem>,
) -> Response {
    match validator::Validate::validate(&body) {
        Ok(_) => (),
        Err(err) => {
            let body =
                DefaultResponse::error(err.to_string().as_str(), err.to_string()).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    }

    let item = match Item::create(
        &db,
        &body.description.unwrap(),
        &body.quantity.unwrap(),
        &body.price.unwrap(),
        &body.tax.unwrap(),
        &body.discount.unwrap(),
        &user_id,
        &invoice_id,
    )
    .await
    {
        Ok(result) => result,
        Err(err) => {
            let body =
                DefaultResponse::error("create item invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("add item to invoice success")
        .with_data(json!(item))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn update_item_to_invoice(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path((_, invoice_id, item_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<RequestAddInvoiceItem>,
) -> Response {
    match validator::Validate::validate(&body) {
        Ok(_) => (),
        Err(err) => {
            let body =
                DefaultResponse::error(err.to_string().as_str(), err.to_string()).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    }

    let item = match Item::update(
        &db,
        &item_id,
        &body.description.unwrap(),
        &body.quantity.unwrap(),
        &body.price.unwrap(),
        &body.tax.unwrap(),
        &body.discount.unwrap(),
        &user_id,
        &invoice_id,
    )
    .await
    {
        Ok(result) => result,
        Err(err) => {
            let body =
                DefaultResponse::error("update item invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("update item to invoice success")
        .with_data(json!(item))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn delete_item_to_invoice(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path((_, invoice_id, item_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Response {
    let item = match Item::delete(&db, &item_id, &invoice_id).await {
        Ok(result) => result,
        Err(err) => {
            let body =
                DefaultResponse::error("delete item invoice failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("delete item to invoice success")
        .with_data(json!(item))
        .into_json();

    (StatusCode::OK, body).into_response()
}
