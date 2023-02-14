use chrono::{NaiveDateTime};
use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;
use crate::utils::default_date_format;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestCreateInvoice {
    pub customer_id: Uuid,
    #[validate(range(min = 10000, max = 10000000))]
    pub amount: i32,
    #[validate(required)]
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(with = "default_date_format")]
    #[validate(required)]
    pub invoice_date: Option<NaiveDateTime>,
}

impl RequestCreateInvoice {
    pub fn to_string_custom_amount(&self, amount: i32) -> String {
        format!(
            "customer_id: {}, total_amount: {}, invoice_date: {:?}",
            self.customer_id, amount, self.invoice_date
        )
    }
}

#[derive(Deserialize, Validate, Debug)]
pub struct RequestAddInvoiceItem {
    #[validate(required)]
    pub description: Option<String>,
    #[validate(required)]
    pub quantity: Option<i32>,
    #[validate(required)]
    pub price: Option<i32>,
    #[validate(required, range(min = 0.0, max = 1))]
    pub tax: Option<f32>,
    #[validate(required, range(min = 0.0, max = 1))]
    pub discount: Option<f32>,
}