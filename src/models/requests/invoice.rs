use chrono::{NaiveDateTime, DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;
use crate::utils::default_date_format;

#[derive(Deserialize, Validate)]
pub struct RequestCreateInvoice {
    pub customer_id: Uuid,
    #[validate(range(min = 0, max = 999999999))]
    pub amount: i32,
    #[serde(with = "default_date_format")]
    pub invoice_date: NaiveDateTime,
}
