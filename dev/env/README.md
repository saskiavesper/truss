# Development Environment

Preconfigured services for local development via Docker Compose.

## Services

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| PostgreSQL | `pgvector/pgvector` | `5432` | Database with pgvector extension |
| RabbitMQ   | `rabbitmq` | `5672` / `15672` | Message broker + management UI |

## Setup

Copy the environment template and adjust values as needed:

```sh
cp dev/env/.env.example .env
```

Edit `.env` to match your local setup (`.env` is gitignored).

## Quick Start

```sh
docker compose up -d
```

Check status:

```sh
docker compose ps
```

Follow logs:

```sh
docker compose logs -f
```

Tear down (preserves volumes):

```sh
docker compose down
```

Reset everything (deletes data):

```sh
docker compose down -v
```

## Connection Details

### PostgreSQL

| Field | Default |
|-------|---------|
| Host | `localhost` |
| Port | `5432` |
| User | `postgres` |
| Password | `postgres` |
| Database | `truss_dev` |

Override via environment variables:

```sh
POSTGRES_USER=admin POSTGRES_PASSWORD=secret docker compose up -d
```

### RabbitMQ

| Field | Default |
|-------|---------|
| AMQP | `localhost:5672` |
| Management UI | `http://localhost:15672` |
| User | `guest` |
| Password | `guest` |

## Environment Variables

All service settings are configurable:

| Variable | Default | Service |
|----------|---------|---------|
| `POSTGRES_HOST` | `localhost` | PostgreSQL |
| `POSTGRES_PORT` | `5432` | PostgreSQL |
| `POSTGRES_USER` | `postgres` | PostgreSQL |
| `POSTGRES_PASSWORD` | `postgres` | PostgreSQL |
| `POSTGRES_DATABASE` | `truss_dev` | PostgreSQL |
| `POOL_SIZE` | `10` | App |
| `RABBITMQ_PORT` | `5672` | RabbitMQ |
| `RABBITMQ_MGMT_PORT` | `15672` | RabbitMQ |
| `RABBITMQ_USER` | `guest` | RabbitMQ |
| `RABBITMQ_PASSWORD` | `guest` | RabbitMQ |
