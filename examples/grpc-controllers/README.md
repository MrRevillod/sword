# grpc-controllers

Minimal gRPC users CRUD example for Sword using `grpc-controllers` and an in-memory store.

## Run

```bash
cargo run -p grpc-controllers
```

## Available services

- `users.UserService`
- `grpc.health.v1.Health`
- `grpc.reflection.v1.ServerReflection` (enabled by config)

## Available RPC methods

- `users.UserService/ListUsers`
- `users.UserService/StreamUsers`
- `users.UserService/CreateUser`
- `users.UserService/GetUser`
- `users.UserService/UpdateUser`
- `users.UserService/DeleteUser`
- `grpc.health.v1.Health/Check`

## Notes

- UserService methods expect `authorization` metadata.
- Server default address is `127.0.0.1:50051`.
- If `application.enable-tonic-reflection = true`, `grpcurl list` includes health and users services.
