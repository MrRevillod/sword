use serde_json::{Map, Value, json};

#[cfg(feature = "validator")]
use validator::ValidationErrors;

#[cfg(feature = "validator")]
/// Structured JSON output for validation errors  from the `validator` crate.
///
/// # Example
///
/// ```json
/// {
///   "email": [
///     {
///       "code": "invalid",
///       "message": "Must be a valid email address"
///     }
///   ],
///   "password": [
///     {
///       "code": "length",
///       "message": "Must be at least 8 characters long"
///     },
///     {
///       "code": "strength",
///       "message": "Must contain a number"
///     }
///   ]
/// }
/// ```
pub fn format_validator_errors(e: ValidationErrors) -> Value {
    let mut formatted_errors = Map::new();

    for (field, field_errors) in e.field_errors() {
        let mut formatted_field_errors = vec![];

        for error in field_errors {
            formatted_field_errors.push(json!({
                "code": error.code,
                "message": error.message,
            }));
        }

        formatted_errors
            .insert(field.to_string(), Value::Array(formatted_field_errors));
    }

    Value::Object(formatted_errors)
}
