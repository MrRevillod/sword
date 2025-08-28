use std::sync::{Arc, OnceLock};

use axum_test::TestServer;
use serde::{Deserialize, Serialize};
use sword::prelude::*;
use sword::web::HttpResult;
use validator::Validate;

pub static APP: OnceLock<Arc<TestServer>> = OnceLock::new();

#[cfg(test)]
fn test_server() -> Result<Arc<TestServer>, Box<dyn std::error::Error>> {
    use sword::application::Application;

    let app = Application::builder()?.controller::<UserController>();

    Ok(APP
        .get_or_init(|| Arc::new(TestServer::new(app.router()).unwrap()))
        .clone())
}

#[derive(Deserialize, Serialize)]
struct Data {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct QueryData {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
struct ValidableQueryData {
    #[validate(range(message = "Page must be between 1 and 1000", min = 1, max = 1000))]
    page: u32,
    #[validate(range(message = "Limit must be between 1 and 100", min = 1, max = 100))]
    limit: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct OptionalQueryData {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Validate)]
struct DefaultValidableQueryData {
    #[validate(range(message = "Page must be between 1 and 1000", min = 1, max = 1000))]
    page: Option<u32>,

    #[validate(range(message = "Limit must be between 1 and 100", min = 1, max = 100))]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ComplexQueryData {
    page: Option<u32>,
    limit: Option<i32>,
    price: Option<f64>,

    search: Option<String>,
    category: Option<String>,

    active: Option<bool>,

    user_name: Option<String>,
    email_filter: Option<String>,
}

#[controller("/users")]
pub struct UserController {}

#[routes]
impl UserController {
    #[get("/simple-query")]
    async fn get_users(ctx: Context) -> HttpResult<HttpResponse> {
        let query: Option<QueryData> = ctx.query()?;

        Ok(HttpResponse::Ok()
            .data(query)
            .message("Users retrieved successfully"))
    }

    #[get("/validate-query")]
    async fn get_users_with_validation(ctx: Context) -> HttpResult<HttpResponse> {
        let query: Option<ValidableQueryData> = ctx.validated_query()?;

        Ok(HttpResponse::Ok()
            .data(query)
            .message("Users retrieved successfully with validation"))
    }

    #[get("/ergonomic-optional-query")]
    async fn get_users_with_ergonomic_query(ctx: Context) -> HttpResult<HttpResponse> {
        let query: OptionalQueryData = ctx.query()?.unwrap_or_default();

        Ok(HttpResponse::Ok()
            .data(query)
            .message("Users retrieved with ergonomic optional query"))
    }

    #[get("/ergonomic-validated-optional-query")]
    async fn get_users_with_ergonomic_validated_optional_query(
        ctx: Context,
    ) -> HttpResult<HttpResponse> {
        let query: DefaultValidableQueryData = ctx.validated_query()?.unwrap_or_default();

        Ok(HttpResponse::Ok()
            .data(query)
            .message("Users retrieved with ergonomic validated optional query"))
    }

    #[get("/complex-query")]
    async fn get_users_with_complex_query(ctx: Context) -> HttpResult<HttpResponse> {
        let query: Option<ComplexQueryData> = ctx.query()?;

        Ok(HttpResponse::Ok()
            .data(query)
            .message("Users retrieved with complex query parameters"))
    }

    #[get("/pattern-match-query")]
    async fn get_users_with_pattern_match(ctx: Context) -> HttpResult<HttpResponse> {
        match ctx.query::<OptionalQueryData>()? {
            Some(query) => Ok(HttpResponse::Ok()
                .data(query)
                .message("Users retrieved with query parameters")),
            None => Ok(HttpResponse::Ok()
                .data(OptionalQueryData::default())
                .message("Users retrieved with default parameters")),
        }
    }
}

#[tokio::test]
async fn unvalidated_query_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/simple-query?page=1&limit=5").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.get("page").is_some());
    assert!(json.data.get("limit").is_some());

    assert_eq!(json.data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(json.data.get("limit").unwrap(), "5".parse::<u32>().unwrap());

    Ok(())
}

#[tokio::test]
async fn validated_query_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/validate-query?page=1&limit=5").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.get("page").is_some());
    assert!(json.data.get("limit").is_some());

    assert_eq!(json.data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(json.data.get("limit").unwrap(), "5".parse::<u32>().unwrap());

    Ok(())
}

#[tokio::test]
async fn validated_query_error_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/validate-query?page=1001&limit=5").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(400_u16, response.status_code().as_u16());

    assert!(json.data.get("type").is_some());
    assert_eq!(json.data.get("type").unwrap(), "ValidationError");
    assert!(json.data.get("errors").is_some());

    let errors = json.data.get("errors").unwrap().as_array().unwrap();

    assert_eq!(errors.len(), 1);

    let error = &errors[0];

    assert!(error.get("field").is_some());
    assert_eq!(error.get("field").unwrap(), "page");
    assert!(error.get("message").is_some());
    assert_eq!(
        error.get("message").unwrap(),
        "Page must be between 1 and 1000"
    );

    Ok(())
}

#[tokio::test]
async fn ergonomic_optional_query_with_params_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app
        .get("/users/ergonomic-optional-query?page=1&limit=5")
        .await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.get("page").is_some());
    assert!(json.data.get("limit").is_some());

    assert_eq!(json.data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(json.data.get("limit").unwrap(), "5".parse::<u32>().unwrap());

    Ok(())
}

#[tokio::test]
async fn ergonomic_optional_query_without_params_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/ergonomic-optional-query").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.get("page").is_some());
    assert!(json.data.get("limit").is_some());

    assert!(json.data.get("page").unwrap().is_null());
    assert!(json.data.get("limit").unwrap().is_null());

    Ok(())
}

#[tokio::test]
async fn ergonomic_validated_optional_query_with_params_test()
-> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app
        .get("/users/ergonomic-validated-optional-query?page=1&limit=5")
        .await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.get("page").is_some());
    assert!(json.data.get("limit").is_some());

    assert_eq!(json.data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(json.data.get("limit").unwrap(), "5".parse::<u32>().unwrap());

    Ok(())
}

#[tokio::test]
async fn ergonomic_validated_optional_query_without_params_test()
-> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/ergonomic-validated-optional-query").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    // Debería retornar valores por defecto
    assert!(json.data.get("page").is_some());
    assert!(json.data.get("limit").is_some());

    Ok(())
}

#[tokio::test]
async fn pattern_match_query_with_params_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/pattern-match-query?page=1&limit=5").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert_eq!(
        json.message.as_ref(),
        "Users retrieved with query parameters"
    );

    Ok(())
}

#[tokio::test]
async fn pattern_match_query_without_params_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;
    let response = app.get("/users/pattern-match-query").await;

    let json = response.json::<ResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert_eq!(
        json.message.as_ref(),
        "Users retrieved with default parameters"
    );

    Ok(())
}

#[tokio::test]
async fn complex_encoded_query_test() -> Result<(), Box<dyn std::error::Error>> {
    use sword::web::ResponseBody;

    let app = test_server()?;

    let encoded_url = "/users/complex-query?page=1&limit=-10&price=99.99&search=hello%20world&category=electronics%26gadgets&active=true&user_name=john%2Bdoe&email_filter=test%40example.com";

    let response = app.get(encoded_url).await;
    let json = response.json::<ResponseBody>();

    println!("Response status: {}", response.status_code().as_u16());
    println!("Response data: {:#?}", json.data);

    assert_eq!(200_u16, response.status_code().as_u16());

    let data = &json.data;

    if let Some(page) = data.get("page") {
        assert_eq!(page, &1u32);
    }
    if let Some(limit) = data.get("limit") {
        assert_eq!(limit, &-10i32);
    }
    if let Some(price) = data.get("price") {
        assert_eq!(price, &99.99f64);
    }

    if let Some(search) = data.get("search") {
        assert_eq!(search, "hello world");
    }
    if let Some(category) = data.get("category") {
        assert_eq!(category, "electronics&gadgets");
    }
    if let Some(user_name) = data.get("user_name") {
        assert_eq!(user_name, "john+doe");
    }
    if let Some(email_filter) = data.get("email_filter") {
        assert_eq!(email_filter, "test@example.com");
    }

    if let Some(active) = data.get("active") {
        assert_eq!(active, &true);
    }

    Ok(())
}
