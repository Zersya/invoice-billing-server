use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::logger::Logger;
use crate::models::contact_channel::ContactChannel;
use crate::models::customer::Customer;
use crate::models::customer_contact_channel::CustomerContactChannel;
use crate::models::requests::customer::{RequestCreateCustomer, RequestUpdateCustomer};
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
) -> Response {
    let customers = match Customer::get_by_merchant_id(&db, &merchant_id).await {
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
    let mut db_transaction = db.begin().await.expect("Failed to begin transaction");

    let customer =
        match Customer::create_using_transaction(&mut db_transaction, &body.name, &merchant_id)
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

    match CustomerContactChannel::create_using_transaction(
        &mut db_transaction,
        &customer.id,
        &body.contact_channel_id,
        &body.contact_channel_value,
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
    let customer = match Customer::update(&db, &customer_id, &body.name, &merchant_id).await {
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
