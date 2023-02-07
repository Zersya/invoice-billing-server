use serde::Deserialize;
use validator_derive::Validate;
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramUpdateItem {
    pub update_id: i64,
    pub message: TelegramMessage,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: TelegramUser,
    pub chat: TelegramChat,
    pub date: i64,
    pub text: String,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub username: String,
    pub language_code: String,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramChat {
    pub id: i64,
    pub first_name: String,
    pub username: String,
}