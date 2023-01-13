use std::borrow::Cow;

use crate::utils::default_date_format;
use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestSchedule {
    pub job_type: String,
    pub external_id: Uuid,
    pub is_recurring: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    #[validate(custom = "validate_repeat_interval_type")]
    pub repeat_interval_type: Option<String>,
    #[serde(with = "default_date_format")]
    pub start_at: Option<NaiveDateTime>,
    #[serde(with = "default_date_format")]
    pub end_at: Option<NaiveDateTime>,
}
#[derive(Deserialize, Validate, Debug)]
pub struct RequestSetStatusSchedule {
    #[validate(custom = "validate_status_job_schedule")]
    pub status: String,
}

fn validate_repeat_interval_type(
    repeat_interval_type: &str,
) -> Result<(), validator::ValidationError> {
    if repeat_interval_type == "PERMINUTE"
        || repeat_interval_type == "HOURLY"
        || repeat_interval_type == "DAILY"
        || repeat_interval_type == "WEEKLY"
        || repeat_interval_type == "MONTHLY"
    {
        return Ok(());
    }

    let err = validator::ValidationError {
        code: Cow::from("invalid_repeat_interval_type"),
        message: Some(Cow::from(
            "Repeat Interval type must be PERMINUTE, HOURLY, DAILY, WEEKLY or MONTHLY",
        )),
        params: Default::default(),
    };

    return Err(err);
}

fn validate_status_job_schedule(status: &str) -> Result<(), validator::ValidationError> {
    if status == "pending" || status == "in_progress" || status == "completed" || status == "failed"
    {
        return Ok(());
    }

    let err = validator::ValidationError {
        code: Cow::from("status"),
        message: Some(Cow::from(
            "Status Job must be pending, in_progress, completed, or failed",
        )),
        params: Default::default(),
    };

    return Err(err);
}
