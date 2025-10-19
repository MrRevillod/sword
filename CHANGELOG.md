# Sword web framework changelog

## Next Release

### Added

- Added `Non static handlers for controllers` support. Now, controllers must have `&self` as the first parameter in their methods. This allows to use struct fields that are extracted from the application state.

- Added schema validation with feature flags. Now the `validator` crate is included only under `validator` feature flag. This allows users to choose if they want to use `validator` crate or not. If not, you can implement your own trait for validation to the `Request` struct. e.g. with `garde`, `validify`.

- Added `global prefix` support. Now, you can set a global prefix for all routes in the application. This is useful for versioning or grouping routes under a common path.

- Added versioning support on controllers with `version` attribute.

- Added `hot-reload` feature flag to `sword`. This enables hot-reloading of the application during development. It uses the `subsecond` and `dioxus-devtools` crates for hot-reloading functionality. See the examples for usage.

- Non static middlewares support. Now, middlewares can also have `&self` as the first parameter in their methods. This allows to use struct fields that are extracted from the dependency injection system.

- `OnRequest` and `OnRequestWithConfig` traits for creating custom middlewares.

- `#[uses]` macro attribute to apply middlewares to controllers.

- Added injection support for middlewares. Now, middlewares can have dependencies injected from the DI system.

- Added injection support for custom configuration types marked with `#[config]` macro.

- Added `DependencyContainer` struct that allows to register components and providers. This struct is used internally by the DI system to manage dependencies.

### Fixed

- Fixed an issue where the middleware macro was not working correctly with some configuration types.

- Fixed the error messages when some macros failed to compile. Now, the error messages are more descriptive and helpful.

### Changed

- With the latest `axum_responses` release, the `data` field in error responses has been removed and replaced with either `error` or `errors`, depending on your configuration. By default, validation errors will be returned under `errors` fields.

- Changed global state scope. Now its necessary to use DI pattern.

- Changed `Context` by `Request`. This change was made because at the first time `Context` was used to handle request, state, and config together. But now, with the new features added, it was more appropriate to use `Request`.

- Change the purpose of `#[middleware]`. Now, it's used to declare a middleware struct instead of applying middlewares to controllers. To apply middlewares to controllers, use the new `#[uses]` attribute.

## [0.1.8]

### Added

- Added `helmet` feature to `sword`. This allows users to enable security-related HTTP headers for their applications. It's built on top of the `rust-helmet` and `axum-helmet` crates.

- Added built-in `Timeout` middleware based on `tower_http` TimeoutLayer. This middleware allows users to set a timeout duration for requests, enhancing the robustness of their applications. The number of seconds can be configured on the config .toml file at the `application` key. See the examples for usage.

- Added documentation comments to all public functions and structs in the `sword` crate. This improves code readability and helps users understand the functionality of the framework better.

- Added `cookies` feature flag to `sword`. This enables cookie parsing and management. It uses the `tower-cookies` crate for cookie handling. This feature allows users to use Cookies, PrivateCookies, and SignedCookies in their handlers. See the examples for usage.

- Added `multipart` feature flag to `sword`. This enables multipart form data handling using the `multipart` feature flag of `axum` crate. Basically it provides `Multipart` extractor for handling file uploads and form data.

- Added support for axum run with graceful shutdown. This allows the application to handle shutdown signals gracefully, ensuring that ongoing requests are completed before the server stops.

- Added `tower_http` layers support to `middleware macro`. This allows users to easily add middleware layers from the `tower_http` to controllers using the `#[middleware]` attribute.

### Changed

- Move `hot-reload` functionality to another branch due to its constantly changing development state.

- Changed the `serde_qs` dependency to native `axum` query extraction. This simplifies the codebase and reduces dependencies.

- Changed the `#[controller_impl]` macro to `#[routes]`. This change improves clarity and consistency in the codebase.

- Changed the builder pattern for `Application` struct. Now, all the build methods start with `with_` prefix. For example, `with_layer`, `with_controller`, etc. This change enhances code readability and consistency.
