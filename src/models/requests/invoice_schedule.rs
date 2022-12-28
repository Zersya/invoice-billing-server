use std::borrow::Cow;

use crate::utils::default_date_format;
use chrono::NaiveDateTime;
use serde::Deserialize;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestInvoiceSchedule {
    pub is_recurring: bool,
    #[validate(custom = "validate_repeat_interval_type")]
    pub repeat_interval_type: Option<String>,
    #[serde(with = "default_date_format")]
    pub start_at: Option<NaiveDateTime>,
    #[serde(with = "default_date_format")]
    pub end_at: Option<NaiveDateTime>,
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
        message: Some(Cow::from("Repeat Interval type must be PERMINUTE, HOURLY, DAILY, WEEKLY or MONTHLY")),
        params: Default::default(),
    };

    return Err(err);
}
