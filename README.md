# Sword

> <img src="https://avatars.githubusercontent.com/u/228345998?s=200&v=4" align="right" width="120"/>

Structured web framework for rust built on top of axum.  
Designed to build server application with less boilerplate and more simplicity.  
It takes advantage of the tokio and axum ecosystem to bring you performance with nice DX.

> Sword is in active development, expect breaking changes.

## Features

- **Macro-based routing** - Clean and intuitive route definitions
- **JSON-first design** - Built with JSON formats as priority
- **Built-in validation** - Support `validator` crate and extensible validation system
- **HTTP responses standarization** - Consistent response formats out of the box
- **Dependency Injection** - Built-in DI support with declarative macros
- **Interactive CLI** - Built to improve the developer experience
- **Real time support** - Built-in support for SocketIO with `socketioxide` crate

## Examples

- [Rest API](./examples/rest_adapter)
- [SocketIO Adapter Chat](./examples/socketio_adapter)
- [Interceptors (Both adapters)](./examples/interceptors)

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for more details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request. See [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.
