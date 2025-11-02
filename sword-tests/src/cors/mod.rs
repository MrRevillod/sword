use sword::core::Application;

#[tokio::test]
async fn test_cors() {
    Application::builder().build();
}
