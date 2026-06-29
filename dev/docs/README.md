# Development Guide

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) + [Docker Compose](https://docs.docker.com/compose/) for PostgreSQL and RabbitMQ

## Environment

Start required services:

```sh
docker compose -f dev/env/docker-compose.yml up -d
```

See [`dev/env/`](../env/) for service configuration
(ports, credentials, environment variables).

- [mise](https://mise.jdx.dev/) — version manager (manages Elixir, Erlang, lefthook, cocogitto)
- The project pins exact versions in [`mise.toml`](../mise.toml)

## Setup

```sh
mise trust && mise setup
```

This runs: `mise install` → installs Hex + Rebar → `mix deps.get` → installs lefthook hooks.

To run steps individually:

```sh
mise install
mix local.hex --force
mix local.rebar --force
mix deps.get
lefthook install --force
```

## Verification Gates

Every change must pass all gates before reaching `main`. The CI pipeline enforces this, and `lefthook` runs them pre-commit.

| Gate | Command | Enforced by | What it checks |
|------|---------|-------------|----------------|
| Format | `mix format --check-formatted` | CI + lefthook | Code style matches `.formatter.exs` |
| Lint | `mix credo --strict` | CI + lefthook | Zero warnings, all checks enabled |
| Tests | `mix test` | CI + lefthook | All `ExUnit` tests pass |
| Compile | `MIX_ENV=prod mix compile --warnings-as-errors` | CI (release check) | No warnings under prod |
| Static analysis | `mix dialyzer` | CI | Type/spec consistency |
| Coverage (CRAP) | `mix coverage` | CI | No function exceeds CRAP score of 30 |

### Shorthand aliases

```sh
mix lint          # format check + credo
mix coverage      # test with coverage export + crap scoring
mix setup         # deps.get + hex + rebar
mix gen.migration # generate a SQL migration file
```

## Database

PostgreSQL via **Postgrex** (no Ecto) with **pgvector** for embeddings and
native JSON columns. Migrations are raw SQL in `priv/migrations/`.

```sh
mix gen.migration create_workspaces
# Creates priv/migrations/20260126_163000__create_workspaces.sql
```

Postgrex types (with pgvector support) are registered in
[`lib/postgrex_types.ex`](../lib/postgrex_types.ex).

## Schema Validation

**Zoi** handles all boundary validation, replacing Ecto changesets. Schemas are
defined in `lib/truss/schema/`. Key features:

- **Auto-coercion**: `Zoi.integer(coerce: true)` parses string inputs
- **Complex checks**: Piped validators (`|> Zoi.min(1) |> Zoi.max(100)`)
- **No field repetition**: name written once per schema
- **OpenAPI derivation**: `Zoi.to_json_schema/1` generates JSON Schema

See the [workspace schema](../lib/truss/schema/workspace.ex) for examples.

## Event Bus

[`Phoenix.PubSub`](https://hexdocs.pm/phoenix_pubsub/) handles all
domain event broadcasting — no additional abstraction layer needed.

```elixir
Phoenix.PubSub.subscribe(Truss.PubSub, "workspace")
Phoenix.PubSub.broadcast(Truss.PubSub, "workspace", {:event, %{type: "workspace_created"}})
# receive: {:event, %{type: "workspace_created"}}
```

The default adapter uses BEAM's built-in PG2 — no external broker required.
The `Truss.PubSub` process starts in the application supervision tree.
See [`lib/truss/application.ex`](../lib/truss/application.ex).

## Git Workflow

### Branch strategy

Never commit directly to `main`. Always create a feature branch:

```sh
git checkout main
git pull --rebase origin main
git checkout -b feat/my-feature
```

### Pull with rebase

Always rebase when pulling to avoid merge commits:

```sh
git pull --rebase origin main
```

Configure as default:

```sh
git config --global pull.rebase true
```

### Commit convention

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/). Format is enforced by `cog` via `lefthook`.

```text
<type>(<scope>): <description>

[optional body]
```

Types defined in [`cog.toml`](../cog.toml):

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

Examples:

```
feat(workspace): add create workspace command
fix(infra): resolve rabbitmq reconnect loop
docs(dev): add development guide
refactor(workspace): extract validation into separate module
```

To commit manually (lefthook will verify):

```sh
git commit -m "feat(workspace): add archive workspace command"
```

### Squash on merge

Always squash commits when merging PRs. This keeps `main` history linear and readable.

```sh
gh pr merge --squash
```

Or via the GitHub UI: select **Squash and merge**.

## Pre-commit Hooks

[lefthook](https://github.com/evilmartians/lefthook) runs automatically on `git commit`:

```yaml
# From lefthook.yml
pre-commit:
  format:   mix format          # auto-formats and stages
  credo:    mix credo --strict  # lint check
  test:     mix test            # test suite
commit-msg:
  cog:      cog verify --file {1}  # conventional commit check
```

To bypass hooks in an emergency (rare):

```sh
git commit --no-verify -m "wip: temporary"
```

## Architecture Decisions

Architecture Decision Records (ADRs) live in [`dev/docs/adr/`](adr/). Each ADR documents a significant architectural choice with context, decision, and consequences.
