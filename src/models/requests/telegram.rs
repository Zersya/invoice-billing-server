use serde::Deserialize;
use validator_derive::Validate;
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramUpdateItem {
    pub update_id: Option<i64>,
    pub message: Option<TelegramMessage>,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramMessage {
    pub message_id: Option<i64>,
    pub from: Option<TelegramUser>,
    pub chat: Option<TelegramChat>,
    pub date: Option<i64>,
    pub text: Option<String>,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramUser {
    pub id: Option<i64>,
    pub is_bot: Option<bool>,
    pub first_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramChat {
    pub id: Option<i64>,
    pub first_name: Option<String>,
    pub username: Option<String>,
}