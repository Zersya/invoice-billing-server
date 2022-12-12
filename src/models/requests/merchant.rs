use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestCreateMerchant {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
    #[validate(length(min = 4, max = 150))]
    pub description: String,
    pub user_id: Uuid,
}

#[derive(Deserialize, Validate)]
pub struct RequestUpdateMerchant {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
    #[validate(length(min = 4, max = 150))]
    pub description: String,
}