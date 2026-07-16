# ADR-002: Snapshot Strategy

## Status

Draft — pending decision

## Context

In an event-sourced system, the event store is the source of truth, but replaying every event from the beginning to rebuild state is impractical at scale. **Snapshots** are a performance optimization: a cached copy of an aggregate's state at a known version, so you only replay events *since* that snapshot rather than from the beginning.

We need to decide **where and when** snapshots are written. Three competing strategies exist:

| Strategy | Who writes snapshots | When | Where state lives between commands |
|----------|-------------------|------|-----------------------------------|
| **A — Snapshot in commit** | Command handler (synchronously) | On every `commit_events` | Disk (snapshot column family) — loaded fresh each time |
| **B — Traditional CQRS** | Read-side projector (asynchronously) | On projector tick | Disk (projector's read model) — command side replays from events |
| **C — In-memory cache + background flush** | Background worker (asynchronously) | Periodic / event-count threshold | Memory (hot cache) — cold start via snapshot + events |

---

## Strategy A — Snapshot in Commit (Previous Roadmap)

### How it works

```
handle(cmd):
    load_state(id):
        snapshot = read_snapshot_from_disk(id)       ← IO
        events = read_events_since(id, snapshot.ver)  ← IO
        state = fold(events, snapshot)
        return state

    reduce(&state, cmd) → (new_state, events)         ← pure

    commit_events(id, events, new_state):
        append_events_to_store(id, events)             ← IO
        write_snapshot_to_disk(id, new_state)          ← IO
        publish(events)                                ← IO
```

### Characteristics

| Aspect | Detail |
|--------|--------|
| **Snapshot freshness** | Always up-to-date — written atomically with events |
| **Cold-start latency** | 1 disk read (snapshot) + potentially many event reads |
| **Hot-path latency** | Every handler does **2 disk reads + 2 disk writes** |
| **Write amplification** | Every single event append also writes a full snapshot |
| **Crash recovery** | Snapshot may be ahead of events (if events failed to append but snapshot written) — **needs careful ordering: append events first, snapshot second** |
| **Read-model consistency** | Snapshot is authoritative for command side — no dependency on query side |

### Why we initially chose it

Simple to reason about. Everything happens in one synchronous flow. The snapshot is guaranteed to be consistent with the event stream if we order writes correctly (events first, snapshot second). No background processes to manage.

### Why it's uncomfortable

Every command pays the cost of writing a full snapshot to disk. For a high-throughput system handling thousands of commands per second, this is significant write amplification. A task that emits a single `TaskCreated` event also writes the entire `Task` struct as a snapshot — redundant when the event is itself just a few bytes.

---

## Strategy B — Traditional CQRS (Read-Side Snapshots)

This is the pattern most CQRS/ES literature describes. The **command side** has no snapshots at all — it replays all events from scratch, or relies on an external read model.

### How it works

```
COMMAND SIDE                          READ SIDE (projector)
┌──────────────────┐                  ┌─────────────────────┐
│ handle(cmd):     │                  │ on_event(event):    │
│   events =       │     events       │   state =           │
│     load_all(id) │ ◄─────────────── │   fold(state,event) │
│     ← IO         │                  │   write_state(state)│
│   state =        │                  │     ← IO            │
│     fold(events) │                  └─────────────────────┘
│     ← pure       │                          │
│   reduce(state)  │                    snapshot written
│     → events     │                    as read model
│   append(events) │
│     ← IO         │
└──────────────────┘
```

When the command side needs state, it either:
1. **Replays all events from the event store** (no snapshot at all — works but costly for long streams)
2. **Reads from the read model** (the projector's output) — creating a **circular dependency** (command side waiting on query side)

### Characteristics

| Aspect | Detail |
|--------|--------|
| **Command-side snapshots** | None — or relies on read models |
| **Command-side latency** | Full replay on every command (or read-model fetch, which is a network hop in multi-node setups) |
| **Read-side writes** | Projector writes state on every event — similar write amplification to Strategy A |
| **Dependency direction** | Command side depends on read side for state (uncomfortable reversal) |
| **Staleness risk** | Read model may lag behind event store → command side reads stale state |
| **Crash recovery** | Trivial — events are authoritative, read model rebuilds from scratch |

### Why it's the "traditional" approach

CQRS literature assumes the command side is lightweight (just validate + emit) and most reads go through the query side. Snapshots on the command side are seen as premature optimization — you just replay events.

### Why it doesn't fit Truss well

Truss's command side is not lightweight. A `MarkCompleteForProject` command needs to check all linked projects, verify dependencies, enforce role permissions, and manage the completion saga. That's a lot of state to carry. Replaying thousands of events for every such command would be wasteful, and depending on a potentially-stale read model is architecturally uncomfortable.

---

## Strategy C — In-Memory Cache + Background Flush (Proposed)

### How it works

```
                        ┌──────────────────────┐
                        │    IN-MEMORY CACHE    │
                        │  ┌────┐ ┌────┐ ┌───┐ │
                        │  │T1  │ │T2  │ │T3 │ │
                        │  └────┘ └────┘ └───┘ │
                        └──────────┬───────────┘
                                   │ subscribes to own events
                                   │ (keeps cache warm)
                        ┌──────────▼───────────┐
                        │      HANDLER          │
                        │ reads from cache      │
                        │ no disk I/O (hot hit) │
                        └──────────┬───────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │ on commit          │ on cache miss      │
              ▼                    ▼                    ▼
     ┌────────────────┐  ┌──────────────────────────┐
     │ 1. append events│  │ 1. read snapshot ← IO   │
     │ 2. update cache │  │ 2. read events since ← IO│
     │ 3. publish      │  │ 3. fold → state         │
     └───────┬────────┘  │ 4. insert in cache       │
             │           └──────────────────────────┘
             │
             │    ┌──────────────────────────────┐
             └───►│  BACKGROUND WORKER (async)    │
                  │  every 5s / every 100 events: │
                  │  for each dirty cache entry:  │
                  │    write_snapshot_to_disk(id) │
                  └──────────────────────────────┘
```

#### Handler flow (hot path — cache hit)

```rust
pub async fn handle_update<C>(ctx: &C, cmd: UpdateOrganization) -> Result<(), UpdateError>
where
    C: HasCache + HasEventCommitter,
{
    // 1. Load from in-memory cache — NO DISK I/O
    let state = ctx.load_cached_state(cmd.id).await?  // cache hit
        .ok_or(UpdateError::NotFound)?;

    // 2. Pure core (no IO)
    let (new_state, uncommitted) = reduce_organization(&state, cmd)?;

    // 3. Commit: events to store, update cache, publish — NO snapshot write
    ctx.commit_events(cmd.id.into(), uncommitted,
        new_state, state.version, ...).await?;
    Ok(())
}
```

#### Handler flow (cold path — cache miss)

```rust
// load_state implementation when cache has no entry:
async fn load_state(id: OrganizationId) -> Result<Organization, Error> {
    // Fall through to disk
    let (snapshot, snap_version) = read_snapshot("organization", id).await?;
    let events = read_events_from("organization", id, snap_version).await?;
    let state = events.iter()
        .map(|e| deserialize::<OrganizationEvent>(&e.data))
        .fold(snapshot.unwrap_or_default(), |s, e| apply_organization_event(&s, &e));

    // Warm the cache
    cache.insert(id, state.clone(), state.version);
    Ok(state)
}
```

#### Background worker

```rust
/// Runs as a long-lived tokio task.
pub async fn snapshot_flusher<C: HasSnapshotStore + HasCache>(ctx: &C) {
    loop {
        // Wait for either N events to pass or M seconds to elapse
        tokio::select! {
            _ = ctx.wait_for_events(100) => {}  // batch size trigger
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}  // time trigger
        }

        for entry in ctx.dirty_cache_entries() {
            ctx.write_snapshot(
                entry.aggregate_type,
                entry.id,
                &entry.state,
                entry.version,
            ).await;
            ctx.mark_clean(entry.id);
        }
    }
}
```

### Cache invalidation and consistency

**Scenario:** Handler A and B both fetch Task 1. A commits first, B has stale cache.

This is **not a problem** because of optimistic concurrency:
1. Handler A loads state at version 5, commits → version 6, cache updated
2. Handler B still has state at version 5, calls `reduce`, tries to commit with `expected_version = 5`
3. EventStore rejects: `ConcurrencyConflict` (stream is at version 6)
4. Handler B **catches the error**, reloads state (cache now has version 6), re-executes `reduce`, retries commit

The cache can be stale — the concurrency check catches it at commit time.

### Entry lifecycle

```
┌──────────┐    cache hit (handler)     ┌──────────┐
│  CLEAN   │ ──────────────────────────►│  CLEAN   │
│ (persisted)│                          │ (no-op)  │
└────┬─────┘                           └──────────┘
     │ cache miss                               ▲
     │ (load from disk)                         │
     ▼                                          │
┌──────────┐   commit_events            ┌───────┴───┐
│   COLD   │ ──────────────────────────►│   DIRTY   │
│ (not in  │  (state in mem,            │ (not yet  │
│  cache)  │   needs flush)             │ persisted)│
└──────────┘                            └─────┬─────┘
                                              │
                                     snapshot_flusher
                                              │
                                              ▼
                                       ┌──────────┐
                                       │  CLEAN   │
                                       │ (flushed) │
                                       └──────────┘
```

If a dirty entry is evicted from the cache (LRU pressure), it's simply lost — the next access will cold-load from the snapshot + events. No data loss because events are the source of truth.

### Cache size management

```rust
pub struct InMemoryCache {
    store: HashMap<AggregateId, CachedEntry>,
    max_entries: usize,          // e.g. 10_000
    lru: LruCache<AggregateId>,  // tracks access order
}

pub struct CachedEntry {
    state: Box<dyn Any + Send>,
    version: Version,
    is_dirty: bool,
    last_accessed: Instant,
}
```

When the cache exceeds `max_entries`, evict the least-recently-used **clean** entry. Dirty entries are never evicted — they're flushed first by the background worker, then evicted.

### Characteristics

| Aspect | Detail |
|--------|--------|
| **Hot-path latency** | **0 disk reads, 1 event append write** — snapshot writes are async |
| **Cold-start latency** | 1 disk read (snapshot) + event replay (same as Strategy A) |
| **Write amplification** | Only event bytes on hot path — full snapshot writes happen in background batches |
| **Crash recovery** | On restart, cache is empty. First access for each aggregate cold-loads from snapshot + events |
| **Cache staleness** | Safe — optimistic concurrency catches stale state at commit time |
| **Memory pressure** | Handled via LRU eviction of clean entries |
| **Multi-node (Raft)** | Only the leader needs the cache. On leadership change, new leader warms cache lazily |
| **Background worker** | Additional component to manage — but simple and self-contained |

---

## Comparison

| Property | A — Snapshot in Commit | B — Traditional CQRS | C — In-Memory + Flush |
|----------|:---:|:---:|:---:|
| **Hot-path disk reads** | 2 (snapshot + events) | N (all events) or 1 (read-model fetch) | **0** |
| **Hot-path disk writes** | 2 (events + snapshot) | 1 (events) | **1** (events only) |
| **Write amplification** | High (full snapshot on every change) | Low for command side | **Low for command side** |
| **Cold-start latency** | Snapshot read + events since | Full replay (all events) or read-model fetch | Snapshot read + events since |
| **Snapshot consistency** | In-band (events-first ordering) | Eventually consistent (projector lag) | Eventually consistent (flush lag) |
| **Command dependency on query side** | None | **Yes** (if reading from read model) | None |
| **Crash safety** | Events-first ordering required | Events are source of truth | Events are source of truth |
| **Implementation complexity** | Low | Medium | Medium |
| **Background processes** | None | Projector(s) | Flush worker + optional projector(s) |

---

## Assessment

### Strategy A is our current baseline

It works correctly, but the synchronous snapshot write on every command is a performance concern. Every `TaskCreated` or `Incremented` event pays the cost of serializing and writing the entire aggregate state to disk. At low throughput it's fine; at high throughput it becomes a bottleneck.

### Strategy B doesn't suit Truss

The command side is too stateful — replaying from scratch on every command is wasteful, and reading from the query side creates an uncomfortable dependency inversion. It also introduces real staleness issues: a projector that's 5 seconds behind means the command side operates on stale data for 5 seconds.

### Strategy C is the most performant

The hot path does exactly one disk write (the event) and zero disk reads. The in-memory cache means aggregate state is accessed at memory speed. Crash safety is preserved because events remain the source of truth. The background flusher is a simple, self-contained component.

**The key insight:** The event store is the source of truth. The in-memory cache and on-disk snapshots are both **derived caches**. They can be lost and rebuilt. The only thing that must survive a crash is the event stream.

### Risks of Strategy C (and mitigations)

| Risk | Mitigation |
|------|------------|
| **Memory exhaustion** if too many aggregates are active | LRU eviction with `max_entries` cap; never evict dirty entries (flush them first) |
| **Background flusher falls behind** under high write volume | Bounded queue depth; if queue exceeds threshold, shed load by skipping snapshot writes for cold entries (they'll cold-load on next access) |
| **Cache inconsistency** between handler instances in multi-threaded setup | Each aggregate is handled by one thread at a time (shard by aggregate_id); optimistic concurrency catches cross-thread races |
| **Warm-up time after restart** | No worse than Strategy A — first access for each aggregate cold-loads. If rapid warm-up is needed, preload active aggregates on startup |
| **Raft leadership change** | New leader starts with empty cache; first few commands per aggregate cold-load (same as any cold start) |

---

## Open Questions for Master

1. **Eviction strategy for the cache** — LRU by last access? Or should we keep frequently-modified aggregates in memory and evict cold ones?

2. **Snapshot format** — Should the snapshot be a full serialization of the state struct (bincode, protobuf), or should it be the last event + a fold marker? Full state is simpler and fast to load.

3. **Flush triggers** — Time-based (every 5s), event-count-based (every 100 events), or both? Both is safest.

4. **Cache pre-warming on startup** — Worth loading the N most-recently-modified aggregates into cache on boot, or lazy-load on first access?

---

*Draft — pending Master's decision*