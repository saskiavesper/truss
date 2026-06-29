# 0001 — Strategy: Rust-to-Elixir Migration

## Status

Accepted

## Context

The Truss platform was originally implemented in Rust as a Cargo workspace
monorepo (`truss_rs/`). To align with team preferences and ecosystem goals, the
codebase is being migrated to Elixir (`truss/`) while preserving the
architectural decisions documented in the original ADRs.

The Rust architecture follows clean architecture / ports-and-adapters with:
- Trait-based CQRS handlers (`CommandHandler`, `QueryHandler`)
- Trait-based repository and event bus abstractions
- Domain primitives with smart-pointer validation (`Deref` + `TryFrom`)
- A scope/actor authorization model
- ArangoDB persistence with RabbitMQ event bus

## Decision

### Elixir Equivalents

| Rust Concept       | Elixir Equivalent       |
|--------------------|-------------------------|
| Traits             | `@behaviour` callbacks  |
| `Result<T, E>`     | `{:ok, value}` / `{:error, reason}` |
| Enum variants      | Tagged tuples / structs |
| `#[async_trait]`   | Native `Task` / `await` |
| `cargo fmt`        | `mix format`            |
| `clippy`           | `credo`                 |
| `figment` config   | `config/*.exs` + app env |
| Mock trait impls   | `mox`                   |
| `arangors`         | `arangox`               |
| `lapin` (RabbitMQ) | `amqp`                  |

### Tooling

- **Formatter**: `mix format` (built-in, consistent with Elixir conventions)
- **Linter**: `credo` with strict mode (mirrors Rust's `clippy::all = deny`)
- **Type analysis**: `dialyxir` (spec-based static analysis)
- **Commit conventions**: `cocogitto` + `lefthook` (copied from Rust setup)
- **CI**: GitHub Actions with format check, credo, test — same pipeline pattern
- **Config**: Elixir `config/*.exs` for compile-time + `config/runtime.exs` for
  env var overrides (replaces `figment`)
- **Version management**: `mise` (same tool, updated to pin Elixir/Erlang)

### Architecture

All original ADRs (001–009) remain valid. The mapping to Elixir is:

- **ADR-001 (DI pattern)**: Behaviours replace trait-based DI; composition
  happens in the supervision tree and application module
- **ADR-002 (generic constraints)**: Behaviour callbacks replace generic
  trait constraints
- **ADR-003 (async signatures)**: Native Elixir `Task.async/await` replaces
  `#[async_trait]`
- **ADR-004 (messaging layers)**: `EventBus` behaviour replaces trait
- **ADR-005 (handler decomposition)**: Private module functions remain the
  decomposition strategy
- **ADR-006 (type aliases)**: `@type` module attributes replace `type` aliases
- **ADR-007 (command variant extraction)**: Structs with `new/1` constructors
- **ADR-008 (validation guarding)**: Changeset-pattern or `new/1` returning
  `{:ok, _} | {:error, _}` replaces smart-pointer coercion
- **ADR-009 (memory footprint)**: Not directly applicable (BEAM manages memory)

### Testing

- `mox` for defining mock behaviour implementations at compile time
- `ExUnit` for test framework (built-in)
- Mock contexts similar to Rust's `MockContext` but generated via `mox`

## Consequences

- **Positive**: Elixir's OTP provides battle-tested supervision, GenServer, and
  fault-tolerance primitives that Rust lacked without additional crates
- **Positive**: `mix format` eliminates formatting bikeshedding
- **Positive**: Runtime config via application env is simpler than figment
- **Negative**: No compile-time generic constraints — behaviour callbacks are
  dynamic dispatch
- **Negative**: `dialyxir` is less precise than Rust's type system
- **Negative**: `arangox` and `amqp` are less mature than their Rust
  counterparts
