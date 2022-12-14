use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultResponse {
    pub status: String,
    pub message: Message,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<String>,
}

impl DefaultResponse {
    pub fn new(status: &str, message: Message) -> Self {
        let status = status.to_string();
        Self {
            status,
            message,
            access_token: None,
            data: None,
            errors: None,
            meta: None,
        }
    }

    pub fn ok(message: &str) -> Self {
        Self::new(
            "ok",
            Message {
                value: message.to_string(),
                debug: None,
            },
        )
    }

    pub fn created(message: &str) -> Self {
        Self::new(
            "created",
            Message {
                value: message.to_string(),
                debug: None,
            },
        )
    }

    pub fn unauthorized(message: &str, debug: Option<String>) -> Self {
        Self::new(
            "unauthorized",
            Message {
                value: message.to_string(),
                debug,
            },
        )
    }

    pub fn error(message: &str, debug: String) -> Self {
        Self::new(
            "error",
            Message {
                value: message.to_string(),
                debug: Some(debug),
            },
        )
    }

    pub fn with_access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_errors(mut self, errors: serde_json::Value) -> Self {
        self.errors = Some(errors);
        self
    }

    //    pub fn with_meta(mut self, meta: serde_json::Value) -> Self {
    //        self.meta = Some(meta);
    //        self
    //    }

    pub fn into_json(self) -> Json<Value> {
        Json(json!(self))
    }
}
