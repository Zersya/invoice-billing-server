use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::Errors;
use crate::logger::Logger;
use crate::models::contact_channel::ContactChannel;
use crate::models::customer::Customer;
use crate::models::customer_contact_channel::CustomerContactChannel;
use crate::models::job_schedule::JobSchedule;
use crate::models::requests::customer::{RequestCreateCustomer, RequestUpdateCustomer, RequestGetCustomers};
use crate::models::responses::DefaultResponse;

pub async fn get_by_authenticated_user(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let customers = match Customer::get_by_merchat_user_id(&db, &user_id).await {
        Ok(customers) => customers,
        Err(err) => {
            let body = DefaultResponse::error("get customers failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get customers by authenticated user success")
        .with_data(json!(customers))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_by_merchant_id(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
    Query(query): Query<RequestGetCustomers>,
) -> Response {

    let tags = match query.tags {
        Some(tags) => {
            if tags.len() > 0 {
                tags.split(",").map(|tag| tag.to_string()).collect()
            }
            else {
                Vec::new()
            }
        }
        None => Vec::new(),
    };

    let customers = match Customer::get_by_merchant_id(&db, &merchant_id, &tags).await {
        Ok(customers) => customers,
        Err(err) => {
            let body = DefaultResponse::error("get customers failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get customers by merchant id success")
        .with_data(json!(customers))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn create(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path((merchant_id,)): Path<(Uuid,)>,
    Json(body): Json<RequestCreateCustomer>,
) -> Response {
    let (name, tags, contact_channel_id, contact_channel_value) =
        match validator::Validate::validate(&body) {
            Ok(_) => (
                body.name.unwrap(),
                body.tags.unwrap(),
                body.contact_channel_id.unwrap(),
                body.contact_channel_value.unwrap(),
            ),
            Err(err) => {
                
                let value = Errors::into_string(err);

                let body = DefaultResponse::error(
                    value.as_str(),
                    "".to_string(),
                )
                .into_json();
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        };

    let mut db_transaction = db.begin().await.expect("Failed to begin transaction");

    let customer =
        match Customer::create_using_transaction(&mut db_transaction, &name, &tags, &merchant_id)
            .await
        {
            Ok(customer) => customer,
            Err(err) => {
                Logger::new(format!("{:?}", err)).log();

                db_transaction
                    .rollback()
                    .await
                    .expect("Failed to rollback transaction");

                let body =
                    DefaultResponse::error("create customer failed", err.to_string()).into_json();

                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        };

    // remove + in +62 from phone number
    let phone_number = contact_channel_value.replace("+", "");

    // replace first 0 with 62 if phone number start with 0
    let phone_number: String = if phone_number.starts_with("0") {
        let mut phone = phone_number.clone();
        phone.replace_range(0..1, "62");
        phone
    } else {
        phone_number
    };

    match CustomerContactChannel::create_using_transaction(
        &mut db_transaction,
        &customer.id,
        &contact_channel_id,
        &phone_number,
    )
    .await
    {
        Ok(customer_contact_channel) => customer_contact_channel,
        Err(err) => {
            Logger::new(format!("{:?}", err)).log();

            db_transaction
                .rollback()
                .await
                .expect("Failed to rollback transaction");

            let body =
                DefaultResponse::error("create customer failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    db_transaction
        .commit()
        .await
        .expect("Failed to commit transaction");

    let body = DefaultResponse::ok("create customer success")
        .with_data(json!(customer))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn update(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path((merchant_id, customer_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RequestUpdateCustomer>,
) -> Response {
    let (name, tags) = match validator::Validate::validate(&body) {
        Ok(_) => (body.name.unwrap(), body.tags.unwrap()),
        Err(err) => {
            let body =
                DefaultResponse::error(err.to_string().as_str(), err.to_string()).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let customer = match Customer::update(&db, &customer_id, &name, &tags, &merchant_id).await {
        Ok(customer) => customer,
        Err(err) => {
            let body =
                DefaultResponse::error("update customer failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("update customer success")
        .with_data(json!(customer))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn delete(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path((merchant_id, customer_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let customer = match Customer::delete(&db, &customer_id, &merchant_id).await {
        Ok(customer) => customer,
        Err(err) => {
            let body =
                DefaultResponse::error("delete customer failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("delete customer success")
        .with_data(json!(customer))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_contact_channels(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
) -> Response {
    let contact_channels = match ContactChannel::get_all(&db).await {
        Ok(contact_channels) => contact_channels,
        Err(err) => {
            let body =
                DefaultResponse::error("get contact channels failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get contact channels success")
        .with_data(json!(contact_channels))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_job_schedule_by_customer(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path((_, customer_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let job_scheduled = match JobSchedule::get_by_job_data_json_by_customer_id(
        &db,
        &customer_id.to_string().as_str(),
    )
    .await
    {
        Ok(job_scheduled) => job_scheduled,
        Err(err) => {
            let body =
                DefaultResponse::error("get job scheduled failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get job scheduled success")
        .with_data(json!(job_scheduled))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_job_schedule_by_authenticated(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let job_scheduled =
        match JobSchedule::get_by_job_data_json_by_user_id(&db, user_id.to_string().as_str()).await
        {
            Ok(job_scheduled) => job_scheduled,
            Err(err) => {
                let body =
                    DefaultResponse::error("get job scheduled failed", err.to_string()).into_json();

                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        };

    let body = DefaultResponse::ok("get job scheduled success")
        .with_data(json!(job_scheduled))
        .into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_tags_by_merchant_id(
    State(db): State<PgPool>,
    Extension(_): Extension<Uuid>,
    Path(merchant_id): Path<Uuid>,
) -> Response {
    let tags = match Customer::get_tags_by_merchant_id(&db, &merchant_id).await {
        Ok(tags) => tags,
        Err(err) => {
            let body = DefaultResponse::error("get tags failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get tags success")
        .with_data(json!(tags))
        .into_json();

    (StatusCode::OK, body).into_response()
}
