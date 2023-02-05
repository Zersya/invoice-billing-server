use serde::Deserialize;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestCreateMerchant {
    #[validate(required, length(min = 4, max = 24))]
    pub name: Option<String>,
    #[validate(required, length(min = 4, max = 150))]
    pub description: Option<String>,
    #[validate(length(min = 4, max = 150))]
    pub address: Option<String>,
    pub phone_country_code: Option<String>,
    #[validate(length(min = 11, max = 15))]
    pub phone_number: Option<String>,
    #[validate(range(min = 0.0, max = 1))]
    pub tax: Option<f32>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct RequestUpdateMerchant {
    #[validate(required, length(min = 4, max = 24))]
    pub name: Option<String>,
    #[validate(required, length(min = 4, max = 150))]
    pub description: Option<String>,
    #[validate(length(min = 4, max = 150))]
    pub address: Option<String>,
    pub phone_country_code: Option<String>,
    #[validate(length(min = 11, max = 15))]
    pub phone_number: Option<String>,
    pub tax: Option<f32>,
}