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

## Database Migrations (Atlas)

This project uses [Atlas](https://atlasgo.io) for database schema migrations.
Atlas is a language-independent CLI tool — install it via:

```sh
# macOS/Linux
curl -sSf https://atlasgo.sh | sh

# or via Homebrew
brew install ariga/tap/atlas
```

### Workflow

The desired database state is defined in `priv/schema.sql`.
When you change the schema, generate a new migration:

```sh
# Reads src, dev, and migration.dir from atlas.hcl's "dev" env:
atlas migrate diff <migration_name> --env dev --format '{{ sql . "  " }}'

# Or explicitly:
atlas migrate diff <migration_name> \
  --to file://priv/schema.sql \
  --dev-url "postgres://postgres:postgres@localhost:5432/truss_dev?search_path=public&sslmode=disable" \
  --format '{{ sql . "  " }}'
```

Apply pending migrations to your local database:

```sh
atlas migrate apply --env dev
```

Validate migration directory integrity:

```sh
atlas migrate validate --env dev
```

> **Note**: `atlas migrate lint` (for detecting destructive changes) is an
> [Atlas Pro](https://atlasgo.io/pricing) feature. The community edition
> validates integrity via `atlas migrate validate` and the `atlas.sum` hash
> file.

### `atlas.hcl`

Project configuration lives in `atlas.hcl` at the project root. It defines
environment connection URLs, the migration directory (`priv/migrations/`),
and the desired schema source (`priv/schema.sql`).

### Dev Database

Atlas needs a **dev database** to parse and validate the SQL schema when
computing diffs. A disposable Docker container is spun up automatically —
no local database needed. The `dev` attribute in `atlas.hcl` uses the
Docker driver with the same pgvector image as docker-compose:

```
docker+postgres://pgvector/pgvector:0.8.3-pg18-trixie/dev?search_path=public
```

Atlas pulls the image, starts the container, computes the diff, and tears
it down — all in one command. No need to start docker-compose first.

See the [Atlas docs](https://atlasgo.io/versioned/intro) for more.

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
