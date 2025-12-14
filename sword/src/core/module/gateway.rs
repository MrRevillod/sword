use crate::core::State;
use axum::Router;
use parking_lot::RwLock;

pub(crate) type WithStateRouterBuilder = Box<dyn Fn(State) -> Router>;

/// Represents the different kinds of gateways that can be registered.
/// Each variant may hold specific builder functions.
///
/// - Rest: The base for RESTful APIs, Axum Router with state.
/// - WebSocket: A socketio layer based gateway, Axum Router with state.
/// - Grpc: Not implemented yet.
pub enum GatewayKind {
    Rest(WithStateRouterBuilder),
    WebSocket(WithStateRouterBuilder),
    Grpc,
}

/// A trait for defining gateways in the application.
///
/// Gateways represent different entry points into your application. Controllers
/// automatically implement this trait, allowing them to be registered as REST gateways
/// within modules.
///
/// The `kind()` method should return the appropriate `GatewayKind` variant with the
/// necessary builder function to create the router for that gateway type.
///
/// # Implementing Gateways
///
/// Typically, you don't implement this manually. The `#[controller]` macro automatically
/// implements `Gateway` for your controller type, registering it as a REST gateway.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller("/api/items")]
/// struct ItemsController { /* ... */ }
///
/// // The macro automatically implements Gateway for ItemsController
/// // In your module:
/// fn register_gateways(gateways: &GatewayRegistry) {
///     gateways.register::<ItemsController>();
/// }
/// ```
pub trait Gateway {
    fn kind() -> GatewayKind;
}

/// Registry for managing and storing different gateway kinds.
///
/// `GatewayRegistry` is used within modules to register gateways that define how requests
/// enter the application. Modules call `register::<G>()` during their registration phase
/// to add gateways (typically controllers) to the application.
///
/// The registry uses `RwLock` for thread-safe concurrent access, allowing gateways to be
/// registered during the application setup phase.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// struct MyModule;
///
/// impl Module for MyModule {
///     fn register_gateways(gateways: &GatewayRegistry) {
///         gateways.register::<UserController>();
///         gateways.register::<ProductController>();
///     }
/// }
/// ```
pub struct GatewayRegistry {
    pub(crate) gateways: RwLock<Vec<GatewayKind>>,
}

impl GatewayRegistry {
    /// Creates a new empty gateway registry.
    pub fn new() -> Self {
        Self {
            gateways: RwLock::new(Vec::new()),
        }
    }

    /// Registers a gateway of type `G` by calling its `kind()` method
    /// and storing the resulting `GatewayKind` in the registry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// gateways.register::<MyController>();
    /// ```
    pub fn register<G: Gateway>(&self) {
        self.gateways.write().push(G::kind());
    }
}

impl Default for GatewayRegistry {
    fn default() -> Self {
        Self::new()
    }
}
