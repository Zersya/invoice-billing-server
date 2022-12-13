use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::models::responses::{DefaultResponse, Message};

#[derive(Debug)]
pub struct Errors {
    errors: ValidationErrors,
}

pub type FieldName = &'static str;
pub type FieldErrorCode = &'static str;

impl Errors {
    pub fn new(errs: &[(FieldName, FieldErrorCode)]) -> Self {
        let mut errors = ValidationErrors::new();
        for (field, code) in errs {
            errors.add(field, ValidationError::new(code));
        }
        Self { errors }
    }
}

impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        use validator::ValidationErrorsKind::Field;

        let mut error_message = String::new();

        let mut errors = json!({});
        for (field, field_errors) in self.errors.into_errors() {
            if let Field(field_errors) = field_errors {
                errors[field] = field_errors
                    .clone()
                    .into_iter()
                    .map(|field_error| field_error.code)
                    .collect();

                if let Some(field_error) = field_errors.get(0) {
                    let mut error_params = String::new();

                    for (key, value) in field_error.params.iter() {
                        if key != "value" {
                            error_params.push_str(&format!("{}: {}, ", key, value));
                        }
                    }

                    error_message = format!("Error on {}: {}", field, field_error.code,);

                    if !error_params.is_empty() {
                        error_message.push_str(&format!(" {}", error_params.trim_end()));
                    }

                    if field_errors.len() > 1 {
                        error_message
                            .push_str(format!(" and {} more", field_errors.len() - 1).as_str());
                    }
                }
            }
        }

        let body = DefaultResponse::new(
            "error",
            Message {
                value: error_message,
                debug: None,
            },
        )
        .with_errors(errors);
        let response = Json(json!(body));

        (StatusCode::UNPROCESSABLE_ENTITY, response).into_response()
    }
}

pub struct FieldValidator {
    errors: ValidationErrors,
}

impl Default for FieldValidator {
    fn default() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }
}

impl FieldValidator {
    pub fn validate<T: Validate>(model: &T) -> Self {
        Self {
            errors: model.validate().err().unwrap_or_else(ValidationErrors::new),
        }
    }

    /// Convenience method to trigger early returns with ? operator.
    pub fn check(self) -> Result<(), Errors> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(Errors {
                errors: self.errors,
            })
        }
    }

    pub fn extract<T>(&mut self, field_name: &'static str, field: Option<T>) -> T
    where
        T: Default,
    {
        field.unwrap_or_else(|| {
            self.errors
                .add(field_name, ValidationError::new("can't be blank"));
            T::default()
        })
    }
}
