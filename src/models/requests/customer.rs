use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestCreateCustomer {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
    pub contact_channel_id: Uuid,
    pub contact_channel_value: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct RequestUpdateCustomer {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
}