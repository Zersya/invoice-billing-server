use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct RequestCreateCustomer {
    #[validate(length(min = 4, max = 24), required)]
    pub name: Option<String>,

    #[validate(required)]
    pub tags: Option<Vec<String>>,

    #[validate(required)]
    pub contact_channel_id: Option<Uuid>,
    
    #[validate(required)]
    pub contact_channel_value: Option<String>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct RequestUpdateCustomer {
    #[validate(length(min = 4, max = 24), required)]
    pub name: Option<String>,

    #[validate(required)]
    pub tags: Option<Vec<String>>,
}