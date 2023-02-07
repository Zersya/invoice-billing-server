// {
//     "update_id": 537836436,
//     "message": {
//       "message_id": 1,
//       "from": {
//         "id": 900069535,
//         "is_bot": false,
//         "first_name": "zeinersyad",
//         "username": "zeinersyad",
//         "language_code": "en"
//       },
//       "chat": {
//         "id": 900069535,
//         "first_name": "zeinersyad",
//         "username": "zeinersyad",
//         "type": "private"
//       },
//       "date": 1675648613,
//       "text": "/start",
//       "entities": [
//         {
//           "offset": 0,
//           "length": 6,
//           "type": "bot_command"
//         }
//       ]
//     }
//   }

use serde::Deserialize;
use validator_derive::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct TelegramUpdateResult {
    pub ok: bool,
    pub result: Vec<TelegramUpdateItem>,
}
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
    pub entities: Vec<TelegramMessageEntity>,
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
    #[serde(rename = "type")]
    pub type_: String,
}
#[derive(Deserialize, Validate, Debug)]
pub struct TelegramMessageEntity {
    pub offset: i64,
    pub length: i64,
    #[serde(rename = "type")]
    pub type_: String,
}
