/// Core framework components for application setup and configuration.
///
/// This module contains the fundamental building blocks of a Sword application:
///
/// - [`Application`](core::Application) - The main application struct that manages routing and configuration
/// - [`ApplicationConfig`](core::ApplicationConfig) - Configuration structure for application settings
/// - [`Config`](core::Config) - Configuration management with file and environment variable support
/// - [`State`](core::State) - Thread-safe state container for sharing data across requests
///
/// ## Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// // Create and configure an application
/// let app = Application::builder()
///     .with_controller::<MyController>()
///     .build();
///
/// // Access configuration
/// let config = app.config::<ApplicationConfig>().unwrap();
/// ```
mod application;
mod config;
mod di;
mod state;
mod traits;
mod utils;

pub use di::*;
pub use utils::deserialize_size;

pub use application::*;
pub use config::{Config, ConfigError, ConfigItem, config};
pub use state::{FromState, FromStateArc, State, StateError};

#[doc(hidden)]
pub use config::ConfigRegistrar;
