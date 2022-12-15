use serde::Deserialize;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestCreateMerchant {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
    #[validate(length(min = 4, max = 150))]
    pub description: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct RequestUpdateMerchant {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
    #[validate(length(min = 4, max = 150))]
    pub description: String,
}