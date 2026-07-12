# Truss

B2B project management and orchestration platform.

Truss provides a robust backend service for managing complex business workflows, 
task orchestration, and team collaboration across organizations. Built with Rust 
for performance and reliability.

## Prerequisites

- [mise](https://mise.jdx.dev/) for tool and task management
- [Docker](https://docs.docker.com/get-docker/) for running development services

## Getting Started

### 1. Install tools and setup

```bash
mise install
mise run setup
```

This will:
- Install Rust and other development tools
- Set up git hooks via lefthook
- Copy `.env.example` to `.env` if it doesn't exist

### 2. Start development services

```bash
mise run services.up
```

This starts PostgreSQL (with pgvector) and NATS via Docker Compose.

### 3. Build and run

```bash
mise run build
cargo run
```

## Available Commands

| Command | Description |
|---------|-------------|
| `mise run setup` | Install tools and setup development environment |
| `mise run build` | Build the project |
| `mise run test` | Run tests |
| `mise run lint` | Run clippy linter |
| `mise run format` | Format code with rustfmt |
| `mise run coverage` | Generate test coverage report |
| `mise run services.up` | Start local development services |
| `mise run services.down` | Stop local development services |
| `mise run services.reset` | Reset local database services (deletes all data!) |

## Development Services

| Service | Port | Purpose |
|---------|------|---------|
| PostgreSQL | 5432 | Database with pgvector extension |
| NATS | 4222 / 8222 | Message broker + monitoring UI |

### Environment Variables

Copy `.env.example` to `.env` and adjust as needed:

```bash
cp .env.example .env
```

## Project Structure

```
truss/
├── src/
│   ├── config/       # Configuration management
│   └── validation/   # Input validation
├── api/              # API definitions
├── configs/          # Configuration files
├── deployments/      # Deployment configurations
├── scripts/          # Utility scripts
├── dev/              # Development environment configs
└── test/             # Test fixtures and utilities
```

## License

Licensed under AGPL-3.0 - see [LICENSE](LICENSE)

---
Active development over [Codeberg](https://codeberg.org/saskiavesper/truss)
