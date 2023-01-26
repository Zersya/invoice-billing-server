use std::ops::Add;

use crate::models::customer::Customer;
use crate::models::invoice::Invoice;
use crate::models::job_schedule::JobSchedule;
use crate::models::merchant::Merchant;
use crate::models::requests::job_scheduler::RequestSchedule;
use crate::models::responses::DefaultResponse;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use rust_decimal::prelude::ToPrimitive;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn set_scheduler(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(merchant_id): Path<Uuid>,
    Json(body): Json<RequestSchedule>,
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
            "start_at and end_at is required for recurring job",
            body.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

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
            body.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if start_at < now.naive_utc() {
        let body = DefaultResponse::error(
            format!("start_at must be greater than current time ( {} )", now).as_str(),
            body.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if body.is_recurring && end_at - start_at < chrono::Duration::days(5) {
        let body = DefaultResponse::error(
            "duration must be more than 5 days",
            body.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    if body.is_recurring && body.repeat_interval_type.is_none() {
        let body = DefaultResponse::error(
            "repeat_interval_type is required for recurring job",
            body.to_string(),
        )
        .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let repeat_interval_type = match &body.repeat_interval_type {
        Some(repeat_interval_type) => repeat_interval_type,
        None => "ONCE",
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

    let mut job_schedule: Option<JobSchedule> = None;

    if body.job_type == "send_invoice" {
        job_schedule = match set_invoice_job_schedule(
            &db,
            &user_id,
            &body.external_id.unwrap(),
            &start_at,
            &repeat_interval,
            &repeat_count,
        )
        .await
        {
            Ok(job_schedule) => Some(job_schedule),
            Err(err) => {
                let body = DefaultResponse::error("set invoice scheduler failed", err.to_string())
                    .into_json();

                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        };
    } else if body.job_type == "send_reminder" {

        if body.title.is_none() || body.title.as_ref().unwrap().is_empty() {
            let body = DefaultResponse::error(
                "title is required for send_reminder job",
                body.to_string()
            )
            .into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        } 

        if body.description.is_none() || body.description.as_ref().unwrap().is_empty() {
            let body = DefaultResponse::error(
                "description is required for send_reminder job",
                body.to_string(),
            )
            .into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }

        let external_ids = match body.external_id {
            Some(external_id) => Vec::from([external_id]),
            None => {

                if body.tag.is_none() || body.tag.as_ref().unwrap().is_empty() {
                    let body = DefaultResponse::error(
                        "tag is required for send_reminder job",
                        body.to_string(),
                    )
                    .into_json();

                    return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
                }

                let mut tags = Vec::new();
                tags.push(body.tag.unwrap());

                let customers = match Customer::get_by_merchant_id(&db, &merchant_id, &tags).await {
                    Ok(customers) => customers,
                    Err(err) => {
                        let body = DefaultResponse::error("get customers by tags failed", err.to_string())
                            .into_json();

                        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
                    }
                };

                let mut external_ids = Vec::new();
                for customer in customers {
                    external_ids.push(customer.id);
                }

                external_ids
                
            } 
        };

        let title = body.title.unwrap();
        let description = body.description.unwrap();

        for external_id in external_ids {
            job_schedule = match set_reminder_job_schedule(
                &db,
                &user_id,
                &merchant_id,
                &external_id,
                &start_at,
                &repeat_interval,
                &repeat_count,
                &title,
                &description,
            )
            .await
            {
                Ok(job_schedule) => Some(job_schedule),
                Err(err) => {
                    let body = DefaultResponse::error("set reminder scheduler failed", err.to_string())
                        .into_json();
    
                    return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
                }
            };
        }
    } else {
        let body =
            DefaultResponse::error("job_type is not supported", body.to_string())
                .into_json();

        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let body = DefaultResponse::ok("set invoice scheduler success")
        .with_data(json!(job_schedule))
        .into_json();

    (StatusCode::OK, body).into_response()
}

async fn set_invoice_job_schedule(
    db: &sqlx::PgPool,
    user_id: &Uuid,
    external_id: &Uuid,
    start_at: &chrono::NaiveDateTime,
    repeat_interval: &i64,
    repeat_count: &i64,
) -> Result<JobSchedule, Json<serde_json::Value>> {
    let invoice = match Invoice::get_by_id(&db, &external_id).await {
        Ok(invoice) => invoice,
        Err(err) => {
            return Err(DefaultResponse::error("get invoice failed", err.to_string()).into_json())
        }
    };

    let customer = match Customer::get_by_id(&db, invoice.customer_id, &invoice.merchant_id).await {
        Ok(customer) => customer,
        Err(err) => {
            return Err(DefaultResponse::error("get customer failed", err.to_string()).into_json())
        }
    };

    let merchant = match Merchant::get_by_id(&db, invoice.merchant_id).await {
        Ok(merchant) => merchant,
        Err(err) => {
            return Err(DefaultResponse::error("get merchant failed", err.to_string()).into_json())
        }
    };

    match JobSchedule::create(
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
        Some(*repeat_interval),
        repeat_count.to_i32(),
        repeat_count.to_i32(),
        None,
        "scheduled",
        None,
        None,
    )
    .await
    {
        Ok(job_schedule) => Ok(job_schedule),
        Err(err) => {
            return Err(
                DefaultResponse::error("create job schedule failed", err.to_string()).into_json(),
            )
        }
    }
}

async fn set_reminder_job_schedule(
    db: &sqlx::PgPool,
    user_id: &Uuid,
    merchant_id: &Uuid,
    external_id: &Uuid,
    start_at: &chrono::NaiveDateTime,
    repeat_interval: &i64,
    repeat_count: &i64,
    title: &str,
    description: &str,
) -> Result<JobSchedule, Json<serde_json::Value>> {
    let customer = match Customer::get_by_id(&db, *external_id, &merchant_id).await {
        Ok(customer) => customer,
        Err(err) => {
            return Err(DefaultResponse::error("get customer failed", err.to_string()).into_json())
        }
    };

    let merchant = match Merchant::get_by_id(&db, *merchant_id).await {
        Ok(merchant) => merchant,
        Err(err) => {
            return Err(DefaultResponse::error("get merchant failed", err.to_string()).into_json())
        }
    };

    match JobSchedule::create(
        &db,
        "send_reminder",
        Some(json!({
            "title": title,
            "description": description,
            "customer_id": external_id,
            "customer_name": customer.name,
            "merchant_id": merchant_id,
            "merchant_name": merchant.name,
            "created_by": user_id,
        })),
        &start_at,
        Some(*repeat_interval),
        repeat_count.to_i32(),
        repeat_count.to_i32(),
        None,
        "scheduled",
        None,
        None,
    )
    .await
    {
        Ok(job_schedule) => Ok(job_schedule),
        Err(err) => {
            return Err(
                DefaultResponse::error("create job schedule failed", err.to_string()).into_json(),
            )
        }
    }
}
