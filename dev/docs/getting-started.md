# Getting Started

This guide walks you through setting up the Truss development environment from scratch.

---

## Prerequisites

| Tool | Purpose | Installation |
|------|---------|--------------|
| [Docker](https://docs.docker.com/get-docker/) + Docker Compose | PostgreSQL & RabbitMQ services | [Install Docker](https://docs.docker.com/get-docker/) |
| [mise](https://mise.jdx.dev/) | Go, golangci-lint, lefthook, cocogitto | [Install mise](https://mise.jdx.dev/getting-started.html) |

---

## 1. Clone the Repository

```sh
git clone https://codeberg.org/saskiavesper/truss.git
cd truss
```

---

## 2. Environment Configuration

Copy the environment template and adjust values as needed:

```sh
cp dev/env/.env.example .env
```

The default `.env` configuration:

```sh
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DATABASE=truss_dev
POOL_SIZE=10
```

> **Note:** The `.env` file is gitignored. Never commit secrets to version control.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_HOST` | `localhost` | PostgreSQL host |
| `POSTGRES_PORT` | `5432` | PostgreSQL port |
| `POSTGRES_USER` | `postgres` | PostgreSQL user |
| `POSTGRES_PASSWORD` | `postgres` | PostgreSQL password |
| `POSTGRES_DATABASE` | `truss_dev` | PostgreSQL database name |
| `POOL_SIZE` | `10` | Database connection pool size |
| `RABBITMQ_PORT` | `5672` | RabbitMQ AMQP port |
| `RABBITMQ_MGMT_PORT` | `15672` | RabbitMQ management UI port |
| `RABBITMQ_USER` | `guest` | RabbitMQ user |
| `RABBITMQ_PASSWORD` | `guest` | RabbitMQ password |

---

## 3. Start Development Services

Start PostgreSQL and RabbitMQ via Docker Compose:

```sh
docker compose -f dev/env/docker-compose.yml up -d
```

### Services

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| PostgreSQL | `pgvector/pgvector` | `5432` | Database with pgvector extension |
| RabbitMQ | `rabbitmq` | `5672` / `15672` | Message broker + management UI |

### Verify Services

```sh
# Check container status
docker compose -f dev/env/docker-compose.yml ps

# Follow logs
docker compose -f dev/env/docker-compose.yml logs -f
```

### Service URLs

- **PostgreSQL:** `localhost:5432`
- **RabbitMQ Management:** [http://localhost:15672](http://localhost:15672) (guest/guest)

---

## 4. Install Project Toolchain

```sh
mise trust && mise setup
```

This installs:
- **Go** (version pinned in `mise.toml`)
- **golangci-lint** — Go linter
- **lefthook** — Git hooks manager
- **cocogitto** — Conventional commits enforcer

Then runs:
- `go get ./...` — Download Go dependencies
- `lefthook install --force` — Set up pre-commit hooks

---

## 5. Verify Installation

Run the verification gates to ensure everything is working:

```sh
mise format    # Format Go source files
mise lint       # Run linters
mise test       # Run tests with race detection
```

All commands should pass without errors.

---

## 6. Start Developing

### Project Structure

See [`structure.md`](structure.md) for a complete overview of the project layout.

### Quick Reference

```sh
# Development commands
mise format     # Format code
mise lint       # Lint code
mise test       # Run tests
mise coverage   # Generate coverage report
mise build      # Build all packages

# Database
mise db.reset   # Reset PostgreSQL + RabbitMQ (deletes data!)

# Docker
docker compose -f dev/env/docker-compose.yml up -d      # Start services
docker compose -f dev/env/docker-compose.yml down        # Stop services
docker compose -f dev/env/docker-compose.yml down -v     # Stop & delete volumes
```

---

## Git Workflow

### Branch Strategy

Never commit directly to `main`. Always create a feature branch:

```sh
git checkout main
git pull --rebase origin main
git checkout -b feat/my-feature
```

### Pull with Rebase

Always rebase when pulling to avoid merge commits:

```sh
git pull --rebase origin main
```

Configure as default:

```sh
git config --global pull.rebase true
```

### Commit Convention

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/). Format is enforced by `cog` via `lefthook`.

```text
<type>(<scope>): <description>

[optional body]
```

| Type | Usage |
|------|-------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no logic change |
| `refactor` | Code change, no fix/feature |
| `perf` | Performance improvement |
| `test` | Adding/fixing tests |
| `build` | Build system or dependencies |
| `ci` | CI configuration |
| `chore` | Other non-source changes |
| `revert` | Revert a previous commit |

**Examples:**

```sh
git commit -m "feat(workspace): add create workspace command"
git commit -m "fix(infra): resolve rabbitmq reconnect loop"
git commit -m "docs(dev): add getting started guide"
```

### Squash on Merge

Always squash commits when merging PRs to keep `main` history linear:

```sh
gh pr merge --squash
```

Or via the GitHub/Codeberg UI: select **Squash and merge**.

---

## Pre-commit Hooks

[lefthook](https://github.com/evilmartians/lefthook) runs automatically on `git commit`:

| Hook | Command | What it checks |
|------|---------|----------------|
| format | `mise format` | Go source is properly formatted |
| lint | `mise lint` | Code passes golangci-lint checks |
| test | `mise test` | All tests pass with race detection |
| commit-msg | `cog verify` | Commit message follows conventional format |

**To bypass hooks in an emergency** (use sparingly):

```sh
git commit --no-verify -m "wip: temporary"
```

---

## Database Migrations (Atlas)

This project uses [Atlas](https://atlasgo.io) for database schema migrations.

### Install Atlas

```sh
# macOS/Linux
curl -sSf https://atlasgo.sh | sh

# or via Homebrew
brew install ariga/tap/atlas
```

### Workflow

The desired schema is defined in `priv/schema.sql`. When you change the schema:

```sh
# Generate a new migration
atlas migrate diff <migration_name> --env dev --format '{{ sql . "  " }}'

# Apply pending migrations
atlas migrate apply --env dev

# Validate migration directory
atlas migrate validate --env dev
```

---

## Troubleshooting

### Services won't start

Check if ports are already in use:

```sh
lsof -i :5432
lsof -i :5672
lsof -i :15672
```

### Reset everything

```sh
# Stop containers and delete all data
docker compose -f dev/env/docker-compose.yml down -v

# Restart fresh
docker compose -f dev/env/docker-compose.yml up -d

# Reset database
mise db.reset
```

### Go tools not found

```sh
# Reinstall tools
mise install

# Verify installation
mise ls
```

---

## Next Steps

- Read [`structure.md`](structure.md) to understand the project layout
- Check [`../env/README.md`](../env/README.md) for detailed environment configuration
- See [`../../cog.toml`](../../cog.toml) for commit type definitions
