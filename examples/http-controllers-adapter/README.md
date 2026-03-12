# http-controllers-adapter

This example demonstrates a REST API for user management using Sword's HTTP controller adapter. It includes CRUD operations for users, with features like JSON validation and PostgreSQL integration.

## Running the Example

1. Ensure Docker is installed.
2. Navigate to the example's directory: `cd examples/http-controllers-adapter`
3. Start PostgreSQL: `docker compose up -d`
4. Run the application: `cargo run`

The API will be available at http://localhost:8081/users.
