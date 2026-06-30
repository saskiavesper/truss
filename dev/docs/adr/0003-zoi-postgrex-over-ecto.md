# 0003 — Zoi + Postgrex over Ecto for Schema Validation and Persistence

## Status

Accepted

## Context

The Rust codebase used a clean-architecture boundary-validation pattern: domain
primitives (`NonEmptyString`, `Id`, `Email`, etc.) with `TryFrom` + `Deref` to
enforce invariants at construction time. The Elixir ecosystem standard is Ecto,
which couples schema definition, validation, persistence, and query building
into a single framework.

Ecto's `changeset` pattern has known friction points when the goal is strict
boundary validation without the full ORM:

- **Repeats field names** — the field name appears in `schema`, `cast`,
  `validate_required`, and `validate_length` — three to four repetitions per
  field
- **No auto-coercion** — string-to-integer, string-to-float, and other
  coercions require explicit `cast` or custom validators
- **Complex cross-field validation** — requires custom `validate_*` functions
  that live outside the schema, scattering logic
- **Single output format** — Ecto schemas are tied to database rows; deriving
  JSON Schema, OpenAPI specs, or other representations requires additional
  libraries (e.g., `json_xema`)
- **Heavy dependency** — Ecto pulls in `db_connection`, `decimal`, `telemetry`,
  and a full query DSL even when only validation is needed

## Decision

### Database Layer

Use **Postgrex directly** (without Ecto) for PostgreSQL access, augmented with
**pgvector** for vector embeddings and native JSON columns.

- SQL migrations managed manually via `mix gen.migration`
- Raw SQL queries via `Postgrex.query/4` — explicit, no magic
- `pgvector` registered as a Postgrex extension for `vector` type support
- JSON columns use PostgreSQL's native `jsonb` type with Elixir maps/`Jason`

### Validation Layer

Use **Zoi** for all boundary validation — request parsing, command validation,
and domain primitive construction.

- Zoi schemas replace Ecto changesets
- Schemas are pure data definitions, not coupled to persistence
- `Zoi.to_json_schema/1` derives OpenAPI-compatible JSON Schema directly
- Coercion is a first-class concept (`Zoi.integer(coerce: true)`)
- Custom types handle domain-specific validation (UUID, vector, JSON)
- Pipe-friendly: `Zoi.string() |> Zoi.min(1) |> Zoi.max(100)`

### Why Not Ecto

| Concern | Ecto | Zoi + Postgrex |
|---------|------|----------------|
| Field name repetition | 3–4× per field | 1× per field |
| Auto-coercion | Manual `cast` per field | Built-in (`coerce: true`) |
| Cross-field validation | Custom `validate_*` functions | Pipe combinators |
| JSON Schema / OpenAPI | Needs `json_xema` or similar | `Zoi.to_json_schema/1` built-in |
| Complex business rules | Changeset + custom validators | `Zoi.custom/2` + pipe |
| Weight | Heavy (Ecto + adapter + decimal) | Light (Zoi: ~50KB) |
| DB coupling | Schema tied to table shape | Schema independent |

### Migration Strategy

Database migrations are managed by **Atlas** (https://atlasgo.io), a
language-independent schema migration tool. The desired database state is
defined declaratively in `schema.sql` at the project root. Atlas auto-generates
versioned migration files in `priv/migrations/` by diffing the desired state
against the current migration chain.

Key commands:

```sh
# Generate a new migration after changing priv/schema.sql:
atlas migrate diff <name> --env dev --format '{{ sql . "  " }}'

# Apply pending migrations:
atlas migrate apply --env dev

# Validate migration directory integrity:
atlas migrate validate --env dev
```

See the [Atlas docs](https://atlasgo.io/versioned/intro) for full workflow
details.

### Postgrex Types

Custom Postgrex types defined in `lib/postgrex_types.ex`:

```elixir
Postgrex.Types.define(Truss.PostgrexTypes, Pgvector.extensions(), [])
```

## Consequences

- **Positive**: Validation is decoupled from persistence — schemas can be used
  in API layers, commands, and queries without loading Ecto
- **Positive**: OpenAPI docs derived directly from validation schemas via
  `Zoi.to_json_schema/1`, eliminating drift between spec and code
- **Positive**: SQL remains explicit — no query DSL abstraction, no N+1
  surprises from preloads
- **Positive**: Lighter dependency tree — avoids Ecto's transitive deps
- **Negative**: No built-in migrations runner — must write raw SQL and track
  migrations manually (or add a lightweight runner later)
- **Negative**: No query builder — complex queries are hand-written SQL
- **Negative**: No changeset-style error rendering for Phoenix forms — Zoi has
  its own error format (`Zoi.treefy_errors/1`)
