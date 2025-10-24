
# Dependency injection example with SQLx

To run this example, make sure you have Docker installed and running on your machine.

## Setup

1. Clone the Sword repository if you haven't already:

   ```bash
    git clone https://github.com/sword-web/sword.git
    cd sword/examples/dependency-injection
    ```

2. Run the PostgreSQL database using Docker Compose:

   ```bash
   docker-compose up -d
   ```

3. Run the Sword application:

   ```bash
   cargo run
   ```

## Endpoints

### List tasks

```bash
curl http://localhost:8080/tasks
```

### Create a new task (with default values)

```bash
curl -X POST http://localhost:8080/tasks
```