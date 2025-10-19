use garde::{Report, Validate};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use sword::prelude::*;

pub trait GardeRequestValidation {
    fn body_garde<T: DeserializeOwned + Validate>(&self) -> Result<T, HttpResponse>
    where
        <T as Validate>::Context: Default;
}

impl GardeRequestValidation for Request {
    fn body_garde<T: DeserializeOwned + Validate>(&self) -> Result<T, HttpResponse>
    where
        <T as Validate>::Context: Default,
    {
        let body = self.body::<T>()?;

        body.validate()
            .map_err(|e| to_http_response("Request body validation failed", e))?;

        Ok(body)
    }
}

fn to_http_response(message: &str, e: Report) -> HttpResponse {
    HttpResponse::BadRequest()
        .message(message)
        .errors(format_garde_errors(e))
}

/// Following the RFC 9457 (Problem Details for HTTP APIs)
/// formats validation errors from the `garde` crate into a structured JSON object.
///
/// ## Important Note
/// Garde does not provide error codes for its validation errors.
/// This is optional, just an example implementation of how to format the errors.
///
/// ## Example
///
/// ```json
/// {
///   "email": [
///     {
///       "message": "Must be a valid email address"
///     }
///   ],
///   "password": [
///     {
///       "message": "Must be at least 8 characters long"
///     },
///   ]
/// }
/// ```
pub fn format_garde_errors(report: Report) -> Value {
    let mut formatted_errors = Map::new();

    for (path, err) in report.into_inner() {
        let key = path.to_string();
        let entry = formatted_errors
            .entry(key)
            .or_insert_with(|| Value::Array(vec![]));

        if let Value::Array(arr) = entry {
            arr.push(json!({
                "message": err.message()
            }));
        }
    }

    Value::Object(formatted_errors)
}
