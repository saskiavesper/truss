# Project Structure

This document describes the directory layout for the `truss` Go project.

## Directory Layout

```
truss/
├── cmd/              # Main application entry points
├── internal/         # Private application code
├── pkg/              # Public library code
├── api/              # API definitions
├── configs/          # Configuration files
├── scripts/          # Build and utility scripts
├── deployments/      # Deployment configurations
├── test/             # Integration and end-to-end tests
└── docs/             # Documentation
```

## Directory Descriptions

### cmd/

Contains the main packages for the application. Each subdirectory represents a separate binary or entry point.

```
cmd/
├── server/           # Main API server
│   └── main.go
└── cli/              # Command-line tool (optional)
    └── main.go
```

**When to add here:** When creating a new executable binary.

---

### internal/

Private application code that cannot be imported by other Go modules. This is where the core business logic lives.

```
internal/
├── domain/           # Domain models and business entities
├── service/          # Business logic and use cases
├── repository/       # Data access layer
├── handler/          # HTTP/gRPC request handlers
├── middleware/        # Request middleware
└── config/           # Application configuration loading
```

**When to add here:** For any code that is specific to this application and should not be shared.

---

### pkg/

Public library code that can be imported by external projects. Use this for reusable utilities and packages.

```
pkg/
├── validator/        # Custom validation logic
├── logger/           # Structured logging utilities
└── errors/           # Custom error types
```

**When to add here:** Only for code that is genuinely reusable outside this project.

---

### api/

API definitions, specifications, and generated code.

```
api/
├── openapi/          # OpenAPI/Swagger specifications
├── grpc/             # Protocol Buffer definitions
└── graphql/          # GraphQL schemas (if applicable)
```

**When to add here:** When defining API contracts or specifications.

---

### configs/

Configuration files for different environments.

```
configs/
├── default.yaml      # Default configuration
├── development.yaml  # Development overrides
├── production.yaml   # Production overrides
└── test.yaml         # Test configuration
```

**When to add here:** For non-code configuration files (YAML, TOML, JSON).

---

### scripts/

Utility scripts for development, building, and CI/CD.

```
scripts/
├── build.sh          # Build script
├── lint.sh           # Linting script
└── test.sh           # Test runner script
```

**When to add here:** For shell scripts or other automation helpers.

---

### deployments/

Infrastructure and deployment configurations.

```
deployments/
├── docker/           # Dockerfiles and docker-compose files
├── kubernetes/       # Kubernetes manifests
└── terraform/        # Infrastructure as code
```

**When to add here:** For any deployment or infrastructure configuration.

---

### test/

Integration, end-to-end, and acceptance tests.

```
test/
├── integration/      # Integration tests
├── e2e/              # End-to-end tests
└── fixtures/         # Test data and fixtures
```

**When to add here:** For tests that require external dependencies or span multiple packages.

**Note:** Unit tests should live alongside the code they test (e.g., `internal/service/user_test.go`).

---

### docs/

Project documentation.

```
docs/
├── structure.md      # This file
├── architecture.md   # Architecture overview (optional)
└── api.md            # API documentation (optional)
```

**When to add here:** For any project-level documentation.

---

## Naming Conventions

- **Packages:** lowercase, single words (e.g., `handler`, `service`, `repository`)
- **Files:** lowercase with underscores for multi-word names (e.g., `user_repository.go`)
- **Tests:** suffix with `_test.go` (e.g., `user_test.go`)
- **Commands:** lowercase, descriptive names (e.g., `server`, `cli`, `worker`)

## Import Path

All internal packages are imported using the module path:

```
truss/internal/...
truss/pkg/...
```

## References

- [Go Project Layout](https://github.com/golang-standards/project-layout)
- [Effective Go](https://go.dev/doc/effective_go)
