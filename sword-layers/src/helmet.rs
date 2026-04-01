pub use axum_helmet::{
    ContentSecurityPolicy, ContentSecurityPolicyDirective,
    CrossOriginEmbedderPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy,
    Header, HelmetLayer, OriginAgentCluster, ReferrerPolicy,
    StrictTransportSecurity, XContentTypeOptions, XDNSPrefetchControl,
    XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies, XPoweredBy,
    XXSSProtection,
};

/// Middleware that adds various security headers to HTTP responses.
/// Helmet helps protect your application from common web vulnerabilities
/// by setting appropriate HTTP headers.
///
/// To get more information, visit:  
///
/// [axum-helmet docs](https://docs.rs/axum-helmet/latest/axum_helmet/)  
///
/// [axum-helmet crate](https://crates.io/crates/axum-helmet)
pub struct Helmet {
    headers: Vec<Header>,
}

impl Helmet {
    pub const fn builder() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Adds a security header to the Helmet configuration.
    /// You can chain multiple calls to this method to add several headers.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use sword::web::helmet::*;
    ///
    /// let helmet = Helmet::builder()
    ///     .with_header(XContentTypeOptions::nosniff())
    ///     .with_header(XXSSProtection::on())
    ///     .build();
    /// ```
    pub fn with_header<H: Into<Header>>(mut self, header: H) -> Self {
        self.headers.push(header.into());
        self
    }

    /// Builds the Helmet middleware layer.
    /// Once built, the layer can be added to the application using
    /// `ApplicationBuilder::with_layer()`.
    pub fn build(self) -> HelmetLayer {
        HelmetLayer::new(axum_helmet::Helmet {
            headers: self.headers,
        })
    }
}
