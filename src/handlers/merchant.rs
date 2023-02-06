use crate::errors::FieldValidator;
use crate::models::job_schedule::JobSchedule;
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
    let mut extractor = FieldValidator::validate(&body);

    let name = extractor.extract("name", body.name);
    let description = extractor.extract("description", body.description);
    let address = extractor.extract("address", Some(body.address));
    let phone_number = extractor.extract("phone_number", Some(body.phone_number));
    let tax = extractor.extract("tax", Some(body.tax));
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response(),
    }

    let merchant = match Merchant::create(&db, &name, &description, &user_id, address, body.phone_country_code, phone_number, tax).await {
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
    let mut extractor = FieldValidator::validate(&body);

    let name = extractor.extract("name", body.name);
    let description = extractor.extract("description", body.description);
    let address = extractor.extract("address", Some(body.address));
    let phone_number = extractor.extract("phone_number", Some(body.phone_number));
    let tax = extractor.extract("tax", Some(body.tax));
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response(),
    }

    let merchant =
        match Merchant::update(&db, merchant_id, &name, &description, &user_id, address, body.phone_country_code, phone_number, tax).await {
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


pub async fn get_job_schedule_by_merchant_id(
    State(db): State<PgPool>,
    Path((merchant_id,)): Path<(Uuid,)>,
) -> Response {
    let job_schedules = match JobSchedule::get_by_job_data_json_by_merchant_id(&db, merchant_id.to_string().as_str()).await {
        Ok(job_schedules) => job_schedules,
        Err(err) => {
            let body = DefaultResponse::error("get job schedules failed", err.to_string()).into_json();

            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("get job schedules by merchant id success")
        .with_data(json!(job_schedules))
        .into_json();

    (StatusCode::OK, body).into_response()
}