use serde::de::DeserializeOwned;
use validator::Validate;

use crate::web::{Request, RequestError};

pub trait ValidatorRequestValidation {
    fn body_validator<T: DeserializeOwned + Validate>(
        &self,
    ) -> Result<T, RequestError>;

    fn query_validator<T: DeserializeOwned + Validate>(
        &self,
    ) -> Result<Option<T>, RequestError>;

    fn params_validator<T: DeserializeOwned + Validate>(
        &self,
    ) -> Result<T, RequestError>;
}

#[cfg(feature = "validator")]
impl ValidatorRequestValidation for Request {
    /// Deserializes and validates the request body using validation rules.
    ///
    /// This method combines JSON deserialization with validation using the
    /// `validator` crate. It first deserializes the request body and then
    /// runs validation rules defined on the target type.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type to deserialize and validate (must implement `DeserializeOwned + Validate`)
    ///
    /// ### Returns
    ///
    /// Returns `Ok(T)` with the deserialized and validated instance, or
    /// `Err(RequestError)` if there are deserialization or validation errors.
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - The request body is empty (`RequestError::BodyIsEmpty`)
    /// - The JSON is invalid (`RequestError::ParseError`)
    /// - The data fails validation rules (`RequestError::ValidationError`)
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use serde::Deserialize;
    /// use validator::Validate;
    ///
    /// #[derive(Deserialize, Validate)]
    /// struct CreateUserRequest {
    ///     #[validate(length(min = 1, max = 50))]
    ///     name: String,
    ///     
    ///     #[validate(email)]
    ///     email: String,
    ///     
    ///     #[validate(range(min = 13, max = 120))]
    ///     age: u32,
    /// }
    ///
    /// ... asuming you have a controller struct ...
    ///
    /// #[post("/users")]
    /// async fn create_user(&self, req: Request) -> HttpResult {
    ///     let user_data: CreateUserRequest = req.body_validator()?;
    ///     
    ///     // now data is guaranteed to be valid
    ///
    ///     Ok(HttpResponse::Ok().data(user_data))
    /// }
    /// ```
    fn body_validator<T>(&self) -> Result<T, RequestError>
    where
        T: DeserializeOwned + Validate,
    {
        let body = self.body::<T>()?;

        body.validate().map_err(|error| {
            RequestError::ValidatorError("Invalid request body", error)
        })?;

        Ok(body)
    }

    /// Deserializes and validates query parameters using validation rules.
    ///
    /// This method combines query parameter parsing with validation using the
    /// `validator` crate. It first deserializes the query string and then
    /// runs validation rules defined on the target type.
    ///
    /// Since query parameters are optional in HTTP, this method returns
    /// `Option<T>` where `None` indicates no query parameters were present.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type to deserialize and validate (must implement `DeserializeOwned + Validate`)
    ///
    /// ### Returns
    ///
    /// Returns:
    /// - `Ok(Some(T))` with the deserialized and validated query parameters if they exist and are valid
    /// - `Ok(None)` if no query parameters are present in the URL
    /// - `Err(RequestError)` if query parameters exist but fail deserialization or validation
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - Query parameters cannot be parsed (`RequestError::ParseError`)
    /// - The data fails validation rules (`RequestError::ValidationError`)
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use serde::Deserialize;
    /// use validator::Validate;
    ///
    /// #[derive(Deserialize, Validate, Default)]
    /// struct SearchQuery {
    ///     #[validate(length(min = 1, max = 100))]
    ///     q: Option<String>,
    ///     
    ///     #[validate(range(min = 1, max = 1000))]
    ///     page: Option<u32>,
    ///     
    ///     #[validate(range(min = 1, max = 100))]
    ///     limit: Option<u32>,
    /// }
    ///
    /// ... asuming you have a controller struct ...
    ///
    /// #[get("/search")]
    /// async fn search(&self, req: Request) -> HttpResult {
    ///     let query: SearchQuery = req.query_validator()?.unwrap_or_default();
    ///     
    ///     Ok(HttpResponse::Ok().data(query))
    /// }
    /// ```
    fn query_validator<T>(&self) -> Result<Option<T>, RequestError>
    where
        T: DeserializeOwned + Validate,
    {
        match self.query::<T>()? {
            Some(query) => {
                query.validate().map_err(|error| {
                    RequestError::ValidatorError("Invalid request query", error)
                })?;

                Ok(Some(query))
            }
            None => Ok(None),
        }
    }

    /// Deserializes and validates path parameters using validation rules.
    ///
    /// This method combines path parameter parsing with validation using the
    /// `validator` crate. It first deserializes the path parameters and then
    /// runs validation rules defined on the target type.
    fn params_validator<T: DeserializeOwned + Validate>(
        &self,
    ) -> Result<T, RequestError> {
        let params = serde_json::to_value(self.params.clone()).map_err(|e| {
            RequestError::ParseError("Failed to serialize params", e.to_string())
        })?;

        let deserialized: T = serde_json::from_value(params).map_err(|e| {
            RequestError::ParseError(
                "Failed to deserialize params to the target type",
                e.to_string(),
            )
        })?;

        deserialized.validate().map_err(|error| {
            RequestError::ValidatorError("Invalid request params", error)
        })?;

        Ok(deserialized)
    }
}
