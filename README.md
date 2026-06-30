# Truss

B2B project management and orchestration platform.

## Development

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) + [Docker Compose](https://docs.docker.com/compose/)
- [mise](https://mise.jdx.dev/)

### Setup

```sh
# Start required services (PostgreSQL + RabbitMQ)
docker compose -f dev/env/docker-compose.yml up -d

# Install Elixir/Erlang toolchain
mise trust && mise setup
```

See [`dev/env/`](dev/env/) for service configuration and
[`dev/docs/`](dev/docs/) for the full development guide.

---
- Active development over [Codeberg](https://codeberg.org/saskiavesper/truss)
- Licensed under AGLP see [LICENSE][./LICENSE]
