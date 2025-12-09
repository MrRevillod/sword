use axum_test::TestServer;
use sword::prelude::*;
use sword::web::helmet::*;

#[controller("/test")]
struct HelmetTestController;

#[routes]
impl HelmetTestController {
    #[get("/")]
    async fn index(&self) -> JsonResponse {
        JsonResponse::Ok().message("Hello from helmet test")
    }
}

struct HelmetTestModule;

impl Module for HelmetTestModule {
    type Controller = HelmetTestController;
}

#[tokio::test]
async fn test_basic_security_headers() {
    let helmet = Helmet::builder()
        .with_header(XContentTypeOptions::nosniff())
        .with_header(XFrameOptions::deny())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);

    let headers = response.headers();

    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
}

#[tokio::test]
async fn test_x_content_type_options() {
    let helmet = Helmet::builder()
        .with_header(XContentTypeOptions::nosniff())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
}

#[tokio::test]
async fn test_x_frame_options() {
    let helmet_deny = Helmet::builder().with_header(XFrameOptions::deny()).build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet_deny)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");

    let helmet_sameorigin = Helmet::builder()
        .with_header(XFrameOptions::same_origin())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet_sameorigin)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("x-frame-options").unwrap(),
        "SAMEORIGIN"
    );
}

#[tokio::test]
async fn test_x_xss_protection() {
    let helmet_on = Helmet::builder().with_header(XXSSProtection::on()).build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet_on)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(response.headers().get("x-xss-protection").unwrap(), "1");

    let helmet_off = Helmet::builder().with_header(XXSSProtection::off()).build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet_off)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(response.headers().get("x-xss-protection").unwrap(), "0");
}

#[tokio::test]
async fn test_strict_transport_security() {
    let helmet = Helmet::builder()
        .with_header(StrictTransportSecurity::default())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);

    let hsts_header = response.headers().get("strict-transport-security");

    assert!(hsts_header.is_some());
    assert!(hsts_header.unwrap().to_str().unwrap().contains("max-age"));
}

#[tokio::test]
async fn test_referrer_policy() {
    let helmet = Helmet::builder()
        .with_header(ReferrerPolicy::no_referrer())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("referrer-policy").unwrap(),
        "no-referrer"
    );
}

#[tokio::test]
async fn test_x_dns_prefetch_control() {
    let helmet_off = Helmet::builder()
        .with_header(XDNSPrefetchControl::off())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet_off)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("x-dns-prefetch-control").unwrap(),
        "off"
    );

    let helmet_on = Helmet::builder()
        .with_header(XDNSPrefetchControl::on())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet_on)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("x-dns-prefetch-control").unwrap(),
        "on"
    );
}

#[tokio::test]
async fn test_x_download_options() {
    let helmet = Helmet::builder()
        .with_header(XDownloadOptions::noopen())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("x-download-options").unwrap(),
        "noopen"
    );
}

#[tokio::test]
async fn test_x_powered_by() {
    let helmet = Helmet::builder()
        .with_header(XPoweredBy::new("Sword Framework"))
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("x-powered-by").unwrap(),
        "Sword Framework"
    );
}

#[tokio::test]
async fn test_x_permitted_cross_domain_policies() {
    let helmet = Helmet::builder()
        .with_header(XPermittedCrossDomainPolicies::none())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response
            .headers()
            .get("x-permitted-cross-domain-policies")
            .unwrap(),
        "none"
    );
}

#[tokio::test]
async fn test_cross_origin_embedder_policy() {
    let helmet = Helmet::builder()
        .with_header(CrossOriginEmbedderPolicy::require_corp())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response
            .headers()
            .get("cross-origin-embedder-policy")
            .unwrap(),
        "require-corp"
    );
}

#[tokio::test]
async fn test_cross_origin_opener_policy() {
    let helmet = Helmet::builder()
        .with_header(CrossOriginOpenerPolicy::same_origin())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response
            .headers()
            .get("cross-origin-opener-policy")
            .unwrap(),
        "same-origin"
    );
}

#[tokio::test]
async fn test_cross_origin_resource_policy() {
    let helmet = Helmet::builder()
        .with_header(CrossOriginResourcePolicy::cross_origin())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response
            .headers()
            .get("cross-origin-resource-policy")
            .unwrap(),
        "cross-origin"
    );
}

#[tokio::test]
async fn test_origin_agent_cluster() {
    let helmet = Helmet::builder()
        .with_header(OriginAgentCluster::new(true))
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.headers().get("origin-agent-cluster").unwrap(),
        "?1"
    );
}

#[tokio::test]
async fn test_content_security_policy() {
    let helmet = Helmet::builder()
        .with_header(ContentSecurityPolicy::default())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);

    let csp_header = response.headers().get("content-security-policy");

    assert!(csp_header.is_some());
    assert!(!csp_header.unwrap().to_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_multiple_security_headers() {
    let helmet = Helmet::builder()
        .with_header(XContentTypeOptions::nosniff())
        .with_header(XFrameOptions::deny())
        .with_header(XXSSProtection::on())
        .with_header(ReferrerPolicy::no_referrer())
        .with_header(XDNSPrefetchControl::off())
        .with_header(XDownloadOptions::noopen())
        .with_header(XPermittedCrossDomainPolicies::none())
        .with_header(CrossOriginEmbedderPolicy::require_corp())
        .with_header(CrossOriginOpenerPolicy::same_origin())
        .with_header(CrossOriginResourcePolicy::same_site())
        .with_header(OriginAgentCluster::new(true))
        .with_header(StrictTransportSecurity::default())
        .build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    let headers = response.headers();

    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-xss-protection").unwrap(), "1");
    assert_eq!(headers.get("referrer-policy").unwrap(), "no-referrer");
    assert_eq!(headers.get("x-dns-prefetch-control").unwrap(), "off");
    assert_eq!(headers.get("x-download-options").unwrap(), "noopen");
    assert_eq!(
        headers.get("x-permitted-cross-domain-policies").unwrap(),
        "none"
    );
    assert_eq!(
        headers.get("cross-origin-embedder-policy").unwrap(),
        "require-corp"
    );
    assert_eq!(
        headers.get("cross-origin-opener-policy").unwrap(),
        "same-origin"
    );
    assert_eq!(
        headers.get("cross-origin-resource-policy").unwrap(),
        "same-site"
    );
    assert_eq!(headers.get("origin-agent-cluster").unwrap(), "?1");
    assert!(headers.get("strict-transport-security").is_some());
}

#[tokio::test]
async fn test_empty_helmet() {
    let helmet = Helmet::builder().build();

    let app = Application::builder()
        .with_module::<HelmetTestModule>()
        .with_layer(helmet)
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), 200);
    assert!(response.text().contains("Hello from helmet test"));
}
