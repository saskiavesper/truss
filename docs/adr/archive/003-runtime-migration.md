# ADR-003: Runtime Migration from Rust to Elixir

## Status

Accepted

## Context

Truss was originally planned as a **Rust + RocksDB + Raft** system. The reasoning was: Rust's type safety and performance are excellent for event-sourced systems, RocksDB provides an embedded high-performance KV store, and Raft gives us consensus for multi-node deployments.

However, as the architecture matured, several hidden complexities emerged that put the project's deployability, contributor friendliness, and long-term maintainability at risk.

### Original architecture (Rust + RocksDB + Raft)

```
                   ┌──────────────┐
     HTTP ────────►│  Rust Node   │◄──── Raft ────► Rust Node
                   │  (RocksDB)   │                 (RocksDB)
                   └──────┬───────┘
                          │
                     PostgreSQL? (read models only)
```

**What we were building ourselves:**
- Event store abstraction (append-only log over RocksDB)
- Aggregate lifecycle management (load → fold → reduce → commit)
- In-memory cache + snapshot flusher background worker
- Command routing with sticky routing for aggregate affinity
- Raft consensus layer (leader election, log replication, quorum)
- Projector framework with checkpoint tracking
- Idempotency via RocksDB column family
- SHA-256 checksum chaining for tamper evidence
- Multi-raft sharding (eventually, for write throughput)

### The five hidden complexities

#### 1. Sticky routing for aggregate affinity

Every command for a given aggregate (e.g., `task:abc`) must reach the same Raft leader that owns that aggregate's shard. This requires consistent hashing at the application layer:

```rust
fn route_to_leader(id: &AggregateId) -> NodeId {
    let shard = hash(id) % NUM_SHARDS;
    raft.leader_for_shard(shard)  // multi-raft required at scale
}
```

Not technically difficult, but additional surface area to design, test, and maintain. Mistakes cause "not leader" errors and retry storms.

#### 2. Raft leader bottleneck

One leader handles all writes. Throughput is bounded by one node. To scale, you need **multi-raft** — shard aggregates across independent Raft groups. Each group has its own log, quorum, and leader election. Managing N independent consensus groups adds significant operational complexity.

#### 3. RocksDB backup is painful

Each node has a physically siloed LSM tree. There's no built-in replication at the storage layer (Raft handles replication at the consensus layer, but each RocksDB is independent). To back up:
- `RocksDB::Checkpoint` for consistent snapshots
- Coordination across N nodes for a cluster-wide consistent backup
- Point-in-time recovery means restoring N snapshots and replaying N WALs

Compare this to Postgres where `pg_dump`, WAL archiving to S3, and point-in-time recovery are decades-old solved problems.

#### 4. Layered concurrency models

- **Async Rust (tokio)** manages task concurrency
- **RocksDB** has internal locking (column family locks, write batches)
- **Raft** has its own consensus protocol with timeouts, heartbeats, and retries

These three concurrency models interact in subtle ways. A split-brain, stale read, or deadlock scenario requires understanding all three layers simultaneously.

#### 5. Open-source adoption barrier

To contribute to Truss under the Rust plan:
1. Install Rust toolchain
2. Compile RocksDB C library (or `librocksdb-sys`, which needs Clang and takes minutes)
3. Understand the Raft implementation + the domain logic
4. Run a multi-node cluster locally for testing

For an open-source project aiming for adoption, every one of these steps reduces the contributor funnel.

### What we actually want to build

Truss is a **B2B project management platform** — its competitive advantage is in the domain logic: completion derivation, capture-to-task pipelines, orchestration rules, integration ACLs. The infrastructure (event store, aggregate lifecycle, pub-sub, projectors) is table stakes. We should use existing, well-understood components rather than building them ourselves.

## Decision

We adopt **Elixir on the BEAM** with **Postgres** and the **Commanded** library as our runtime and infrastructure stack.

### New architecture

```
                   ┌─────────────────┐
     HTTP ────────►│   Elixir Node   │◄─── libcluster ────► Elixir Node
                   │  (GenServers)   │                      (GenServers)
                   └────────┬────────┘
                            │
                     ┌──────▼──────┐
                     │  PostgreSQL │
                     │  (single DB)│
                     └─────────────┘
```

**Key components:**

| Component | Technology | Provides |
|-----------|-----------|----------|
| Web framework | Phoenix | HTTP routing, LiveView, WebSockets, JSON serialization |
| CQRS/ES framework | Commanded | Aggregate lifecycle, event store, router, projectors, process managers, idempotency |
| Event store | Commanded.EctoEventStore + Postgres | Append-only event log, optimistic concurrency, global position |
| Read models | Commanded.Projections.Ecto + Postgres | Auto-resuming projectors, checkpoint tracking |
| Sagas | Commanded.ProcessManagers | Cross-aggregate orchestration (completion derivation) |
| Database | Postgres | Events table, snapshots table, read model tables |
| Cluster discovery | libcluster + :pg | Node mesh, process group registration |
| Deploy | Mix release + Docker | Single OTP release, standard Docker image |

### What Commanded provides out of the box

| Concern | We were building in Rust | Commanded provides |
|---------|--------------------------|--------------------|
| Aggregate lifecycle | Handler → load → fold → reduce → commit → publish | `Aggregate` GenServer — auto-loads state, calls `execute/2`, persists events, publishes to bus. Idle timeout kills the process. |
| Event store | RocksDB append-only log with custom schema | `EctoEventStore` — Postgres-backed, versioned streams, optimistic concurrency, global position |
| Command routing | Sticky routing + consistent hashing | `Router.dispatch/2` — routes to the aggregate's GenServer by identity. BEAM serializes messages per process. |
| Idempotency | RocksDB column family for `(idempotency_key, aggregate_id)` | Built-in — dedup by `causation_id`. Second dispatch returns cached events. |
| Event bus | Tokio broadcast channel | In-process PubSub via `:pg`. Subscribers are supervised. |
| Projectors | Custom `Projector` trait, checkpoint column family | `Commanded.Projections.Ecto` — EctoProjector with `projection_versions` table for checkpointing |
| Process managers | Hand-rolled `CompletionSaga` with event subscriptions | `Commanded.ProcessManagers.ProcessManager` — purpose-built for sagas |
| Error handling | `Result<T, E>` everywhere, manual retry | "Let it crash" — OTP supervisors restart on failure. Automatic retry with backoff. |

### What we still own (the valuable parts)

Commanded handles infrastructure. **We** write domain logic:

```elixir
defmodule Truss.Task.Aggregate do
  use Commanded.Aggregate

  # THIS is our competitive advantage — not the event store plumbing
  def execute(%Task{status: :archived}, _cmd), do: {:error, :task_archived}
  def execute(state, %MarkCompleteForProject{}) do
    with :ok <- check_dependencies(state),
         :ok <- check_completion_derivation(state) do
      state
      |> emit(%TaskCompletedInProject{...})
    end
  end

  def apply(%Task{} = state, %TaskCompletedInProject{} = evt) do
    # Update completion tracking
  end
end
```

## Rationale

### Why Elixir over Rust for this project

| Concern | Rust approach | Elixir approach | Winner |
|---------|---------------|-----------------|--------|
| **Concurrency model** | Layered (tokio + RocksDB + Raft) | BEAM actor model — one model, no locks | Elixir |
| **Aggregate isolation** | Sticky routing + Raft sharding | GenServer per aggregate — VM serializes by pid | Elixir |
| **Event store** | Custom RocksDB append-only log | Commanded.EctoEventStore (Postgres, battle-tested) | Elixir |
| **Distributed consensus** | Raft (build + tune ourselves) | Postgres replication + libcluster | Elixir |
| **Backup / recovery** | N RocksDB snapshots | pg_dump / WAL archive / managed RDS | Elixir |
| **Open-source deploy** | Docker with RocksDB compilation | Docker with standard Postgres | Elixir |
| **Contributor onboarding** | Rust + Raft + event sourcing | Elixir + Ecto + familiar Postgres | Elixir |
| **Raw compute perf** | Native, zero-cost abstractions | BEAM VM with garbage collection | Rust |
| **Type safety** | Algebraic types, borrow checker | Dynamic with Dialyzer + pattern matching | Rust |

**The perf argument evaluated:** Truss is I/O-bound (database writes, API calls, LLM requests), not CPU-bound. The BEAM handles millions of GenServers per node. Aggregate operations are small struct manipulations — not compute-intensive. Rust's performance advantage is irrelevant for this workload.

### Why Postgres over RocksDB

| Concern | RocksDB | Postgres |
|---------|---------|----------|
| **Backup** | Node-level checkpoints + coordination | `pg_dump`, WAL archiving to S3, PITR |
| **Replication** | Not built-in (Raft handles it) | Streaming replication, logical replication |
| **Monitoring** | `rocksdb::DB::GetProperty` | `pg_stat_activity`, Datadog, Grafana, every tool |
| **Hosting** | Self-managed only | RDS, Aurora, Supabase, Crunchy Bridge, self-hosted |
| **Schema** | Column families (no schema enforcement) | Tables, migrations, constraints, indexes |
| **Query** | KV get/put/scan | SQL — filters, joins, aggregations |
| **Durability** | WAL + sync writes | WAL + fsync + synchronous replication |
| **Ecosystem** | Limited | Everyone knows how to operate Postgres |
| **Contributor familiarity** | Low | Very high |

### Why Commanded over a custom framework

Building a CQRS/ES framework from scratch in Rust would have been educational but would delay shipping product by months. Commanded is:

- **Mature** — v1.4, used in production by multiple companies
- **Well-documented** — guides, hex docs, examples
- **Extensible** — event store adapter, projector, process manager are all swappable
- **Tested** — `Commanded.TestHelpers` provide test infrastructure
- **Maintained** — active community, regular releases

If Commanded ever doesn't meet our needs, the abstractions are clean enough that we can fork or replace individual components (e.g., swap the event store adapter) without changing domain logic.

## Consequences

### Positive

1. **Phase 0 drops from ~250 lines of custom infrastructure to a setup script.** `mix phx.new` + `mix commanded.event_store.setup` + one aggregate module.

2. **Deploy is `docker compose up`** with a standard Postgres image. No RocksDB compilation, no Raft cluster configuration. Anyone can run Truss locally in 5 minutes.

3. **Backups are trivial.** `pg_dump`, WAL archiving, or managed Postgres. Every DevOps engineer knows how to do this.

4. **Concurrency correctness is guaranteed by the BEAM.** Aggregate GenServers process messages sequentially. No sticky routing, no distributed locks, no contention bugs.

5. **Hot code reloading.** Deploy new code without stopping the VM. Zero-downtime deploys without a complex orchestration layer.

6. **LiveView for real-time UI.** The AI-interactive workspace benefits from LiveView's real-time updates without building separate WebSocket infrastructure.

7. **The FC/IS pattern survives.** Our pure `reduce`/`apply_event` functions map directly to Commanded's `execute/2`/`apply/2`. All the domain modeling we did — commands, events, business rules, process managers — carries over unchanged.

### Negative

1. **Performance ceiling.** For extremely CPU-bound workloads (e.g., real-time video processing, high-frequency trading), Elixir would be the wrong choice. For Truss's I/O-bound workload, the ceiling is more than sufficient.

2. **Less type safety.** Rust's type system catches certain classes of bugs at compile time that Elixir catches at test time. Mitigated by Dialyzer, type specs, and property-based testing.

3. **BEAM memory footprint.** Each GenServer has overhead. Mitigated by idle timeouts — only active aggregates stay in memory.

4. **Framework dependency.** Truss now depends on Commanded's maintenance and release cycle. Mitigated by clean abstractions — we can fork or adapt if needed.

### Migration path

The Rust prototype work (ADR-001 domain architecture, ADR-002 snapshot strategy) informed the design but produced no production code. The migration is **conceptual, not code — we're adopting the same patterns on a different runtime.**

| Artifact | Status | Action |
|----------|--------|--------|
| ADR-001: Domain Architecture | Updated | Already reflects Elixir patterns (behaviours, `execute`/`apply`) |
| ADR-002: State Management | Superseded | Commanded's GenServer lifecycle replaces custom snapshot logic. Aggregate state lives in memory and is rebuilt from events on demand. |
| Development Roadmap | Updated | Phase 0 rewritten for Phoenix + Commanded + Postgres. Domain phases unchanged. |

## Alternatives Considered

### Rust + RocksDB + Raft (original plan)

Rejected due to: sticky routing complexity, Raft leader bottleneck, RocksDB backup pain, layered concurrency model, contributor barrier.

### Rust + Postgres + custom CQRS

We could replace RocksDB with Postgres while keeping Rust. This solves backup/ops but keeps the biggest complexity: building CQRS/ES infrastructure from scratch. Without Commanded's aggregate lifecycle, router, projectors, and process managers, we'd spend months building what Commanded already provides. This is a middle ground that solves the operational problem but not the velocity problem.

### Elixir + RocksDB (via Erlex)

The BEAM has RocksDB bindings (`Erlex`). We could keep RocksDB for the event store while using Elixir for the application layer. Rejected because it combines the worst of both worlds: the operational complexity of RocksDB backup with the concurrency model of the BEAM. Postgres is strictly simpler for our use case.

## References

- Obsidian note: `architecture/truss-runtime-decision.md` (detailed pros/cons comparison)
- ADR-001: Domain Architecture Pattern (FC/IS, value structs, reduce/apply)
- ADR-002: State Management (comparison of three approaches — superseded by Commanded's model)
- Development Roadmap: `projects/Truss — Development Roadmap.md` (updated for Elixir)
- [Commanded documentation](https://hexdocs.pm/commanded/)
- [Commanded.EctoEventStore](https://hexdocs.pm/commanded_ecto_eventstore/)
