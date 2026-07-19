# ADR-002: State Management

## Status

Accepted

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

## Strategy C — Actor-Per-Aggregate (Proposed)

### How it works

Instead of a central cache protected by locks, each aggregate gets its own **tokio task** (worker). A `DashMap<AggregateId, Sender<Msg>>` routes commands to the right worker by aggregate ID. This is the actor-per-aggregate pattern — the BEAM GenServer model, implemented in Rust.

```
                     ┌──────────────────────────────────┐
                     │ AggregateRouter                  │
                     │ DashMap<AggregateId, Sender<Msg>>│
                     └────┬─────────────────────────────┘
                          │ dispatch by aggregate_id
          ┌───────────────┼───────────────────┐
          ▼               ▼                   ▼
   ┌─────────────┐ ┌─────────────┐   ┌─────────────┐
   │ Worker(A,1) │ │ Worker(B,5) │   │ Worker(C,2) │
   │ tokio task  │ │ tokio task  │   │              │
   │             │ │             │   │ (evicted —   │
   │ state       │ │ state       │   │  idle timeout│
   │ version: 12 │ │ version: 3  │   │  exceeded)   │
   │ counter: 12 │ │ counter: 3  │   └─────────────┘
   │ timer: 2s   │ │ timer: 12s  │         │
   │ cmd_rx      │ │ cmd_rx      │         │ re-spawn on
   └─────────────┘ └─────────────┘         │ next dispatch
                                           ▼
                                    ┌─────────────┐
                                    │ Worker(C,2) │
                                    │ (cold-load) │
                                    └─────────────┘
```

Each worker maintains:

| Component | Role |
|-----------|------|
| **State** | The aggregate's current in-memory state + version. Pure domain data, no serialization needed for access. |
| **Message counter** | Incremented on each `Dispatch`. When it hits `SNAPSHOT_INTERVAL` (e.g., 100), the worker flushes a snapshot inline. |
| **Idle timer** | A `tokio::time::Sleep` reset on every command. If it fires (no commands for `IDLE_TIMEOUT`, e.g., 30s), the worker flushes its snapshot and drops — freeing memory. |
| **Shutdown signal** | A `oneshot::Receiver` for external shutdown (SIGTERM). Worker flushes snapshot, cancels timer, exits cleanly. |
| **Command channel** | An `mpsc::Receiver<Msg>` — incoming commands are processed sequentially, one at a time. |

### Worker lifecycle

```
   dispatch(cmd)          dispatch(cmd)          timer expired
   ┌──────┐              ┌──────┐               (no cmd for N secs)
   │ SPAWN│──cmd rx─────►│ ACTIVE│──timeout─────►│ SHUTDOWN
   │ load │  reset timer │ state in mem          │ flush snapshot
   │ state│  inc counter │                       │ drop worker
   └──────┘              └──────┘               └──────────
                               ▲
                          dispatch(cmd)
                          (re-spawn on next access)
```

### Handler flow (inside the worker loop)

```rust
async fn run(&mut self) {
    let mut msg_counter = 0u64;

    loop {
        tokio::select! {
            msg = self.cmd_rx.recv() => {
                match msg {
                    Some(Msg::Dispatch(cmd)) => {
                        msg_counter += 1;

                        // 1. Pure core: reduce command against in-memory state
                        let (new_state, events) = reduce(&self.state, cmd)?;

                        // 2. Commit: append events to store, publish
                        self.commit(events, &new_state).await?;

                        // 3. Update in-memory state
                        self.state = new_state;

                        // 4. Periodic snapshot flush (by message count)
                        if msg_counter % SNAPSHOT_INTERVAL == 0 {
                            self.flush_snapshot().await?;
                        }

                        // 5. Idle timer reset — just loop back to select!
                    }
                    Some(Msg::FlushSnapshot) => {
                        self.flush_snapshot().await?;
                    }
                    Some(Msg::Shutdown) | None => {
                        self.flush_snapshot().await?;
                        break;
                    }
                }
            }
            _ = &mut self.idle_timer => {
                // Idle timeout — flush snapshot and drop
                self.flush_snapshot().await?;
                break;
            }
        }
    }
}
```

### Router — command dispatch

```rust
struct AggregateRouter {
    workers: DashMap<AggregateId, mpsc::Sender<Msg>>,
}

impl AggregateRouter {
    async fn dispatch(&self, id: AggregateId, cmd: impl Into<Msg>) -> Result<()> {
        // Fast path — worker exists
        if let Some(tx) = self.workers.get(&id) {
            tx.send(cmd.into()).await?;
            return Ok(());
        }

        // Cold path — spawn a new worker
        let (tx, rx) = mpsc::channel(256);
        let mut worker = AggregateWorker::load_from_store(id, rx).await?;
        tokio::spawn(async move { worker.run().await });
        self.workers.insert(id, tx);
        self.dispatch(id, cmd).await
    }
}
```

### Snapshot flushing triggers

| Trigger | Condition | Behavior |
|---------|-----------|----------|
| **Message counter** | Every `SNAPSHOT_INTERVAL` (e.g., 100) commands | Synchronous flush inside the worker. No background scanner needed. |
| **Idle timeout** | No commands for `IDLE_TIMEOUT` (e.g., 30s) | Flush snapshot, then drop the worker. Frees memory for cold aggregates. |
| **Graceful shutdown** | Signal received (SIGTERM) | Flush snapshot, cancel timer, drop. |

### Consistency model

Since each aggregate has exactly **one** worker processing commands sequentially via its channel:

- **No concurrent state mutations** — the worker's `state` field is never accessed from two tasks
- **No cache invalidation needed** — the worker IS the cache, and it's the single writer
- **No optimistic concurrency failures within the node** — the channel serializes all commands for a given aggregate

Optimistic concurrency (`expected_version`) still protects against **bug scenarios** where the same aggregate somehow receives concurrent dispatches

### Entry lifecycle

```
                 ┌──────────┐
                 │  SPAWN   │
                 │ load from│
                 │ store    │
                 └────┬─────┘
                      │ worker inserted into DashMap
                      ▼
              ┌──────────────┐
         ┌───►│   ACTIVE     │◄────┐
         │    │              │     │
         │    │ state in mem │     │ cmd received
         │    │ counter: N   │─────┘ (reset timer,
         │    │ timer: T     │      inc counter)
         │    └──────┬───────┘
         │           │
         │           │ idle timeout  │ shutdown signal
         │           │ counter thres.│
         │           ▼               ▼
         │    ┌──────────────┐
         │    │   FLUSH      │
         │    │ write snap   │
         │    │ close chan   │
         │    └──────┬───────┘
         │           │
         │           ▼
         │    ┌──────────────┐
         │    │   DROPPED    │
         │    │ (removed     │
         │    │  from memory)│
         │    └──────────────┘
         │           │
         └───────────┘ (next dispatch re-spawns)
```

### Characteristics

| Aspect | Detail |
|--------|--------|
| **Hot-path latency** | **0 disk reads, 1 event append** — state is in the worker's local memory, no cache lookup |
| **Cold-start latency** | Spawn worker → load snapshot + replay events → first command executes |
| **Snapshot flush latency** | Synchronous inline (counter threshold) or on idle-drop — no background process needed |
| **Write amplification** | Only event bytes on hot path; snapshot writes only at interval or on eviction |
| **Memory pressure** | Controlled via idle timeout — workers drop after `IDLE_TIMEOUT` seconds of inactivity |
| **Concurrency model** | Sequential per channel — no locks, no races within an aggregate |
| **Cache eviction** | Idle timeout → flush → drop. No central LRU scan needed. |
| **Crash recovery** | Worker lost (panic, node restart) → next dispatch cold-loads from event store. Events are source of truth. |
> **Future: Distribution via hash ranges** — When we tackle multi-node deployments, aggregates can be partitioned by hash range (e.g., `hash(aggregate_id) % N`). The router checks which node owns the hash range and forwards the command. The aggregate worker pattern is compatible: each node's router dispatches to its local `DashMap`. This is deferred until distribution is needed.

---

## Comparison

| Property | A — Snapshot in Commit | B — Traditional CQRS | C — Actor-Per-Aggregate |
|----------|:---:|:---:|:---:|
| **Hot-path disk reads** | 2 (snapshot + events) | N (all events) or 1 (read-model fetch) | **0** |
| **Hot-path disk writes** | 2 (events + snapshot) | 1 (events) | **1** (events only) |
| **Write amplification** | High (full snapshot on every change) | Low for command side | **Low for command side** |
| **Cold-start latency** | Snapshot read + events since | Full replay (all events) or read-model fetch | Snapshot read + events since |
| **Snapshot consistency** | In-band (events-first ordering) | Eventually consistent (projector lag) | **Immediate** (flush inline or on drop) |
| **Command dependency on query side** | None | **Yes** (if reading from read model) | None |
| **Crash safety** | Events-first ordering required | Events are source of truth | Events are source of truth |
| **Implementation complexity** | Low | Medium | Medium |
| **Background processes** | None | Projector(s) | **None** — flush inline within the worker |

---

## Assessment

### Strategy A is our current baseline

It works correctly, but the synchronous snapshot write on every command is a performance concern. Every `TaskCreated` or `Incremented` event pays the cost of serializing and writing the entire aggregate state to disk. At low throughput it's fine; at high throughput it becomes a bottleneck.

### Strategy B doesn't suit Truss

The command side is too stateful — replaying from scratch on every command is wasteful, and reading from the query side creates an uncomfortable dependency inversion. It also introduces real staleness issues: a projector that's 5 seconds behind means the command side operates on stale data for 5 seconds.

### Strategy C is the cleanest fit

Each aggregate is an isolated actor (tokio task) with its own state in local memory — no central cache, no locks, no background scanner. Commands arrive via a dedicated channel and are processed sequentially, exactly like BEAM GenServers. The worker flushes snapshots either at a message-count threshold or on idle timeout, then drops itself from memory. No background processes.

**The key insight:** Events are the source of truth. The worker's in-memory state and the on-disk snapshots are both **derived caches** — they can be lost and rebuilt. The worker is ephemeral; the event stream is permanent.

### Risks of Strategy C (and mitigations)

| Risk | Mitigation |
|------|------------|
| **Memory exhaustion** if too many aggregates are active concurrently | Idle timeout per worker (`IDLE_TIMEOUT`). If all aggregates are genuinely active simultaneously, the system is at capacity — scale vertically or shard. |
| **Worker startup latency** on first command after idle period | Cold-load is snapshot read + event replay. Same as Strategy A. Acceptable for infrequent aggregates. |
| **Channel backpressure** if commands arrive faster than a worker can process them | Bounded channel capacity (e.g., 256). Overflow → `try_send` failure → caller retries with backpressure signal. |
| **Timer granularity** for idle timeout | `tokio::time::Sleep` is accurate enough for second-level timeouts. No need for high-precision timers. |

---

## Decisions

### Strategy Chosen: C — Actor-Per-Aggregate (with read-side projectors)

The command side uses the actor-per-aggregate pattern (Strategy C) with the following parameters, as decided by Master.

For **read models**, events are dispatched to the event bus, where background projectors compute read models asynchronously — combining the command-side efficiency of C with the query-side flexibility of traditional CQRS projectors.

### Configuration Parameters

| Parameter | Default | Configurable? | Notes |
|-----------|---------|---------------|-------|
| **`IDLE_TIMEOUT`** | 20s | Yes | Worker drops from memory after this period of inactivity. On next access, a new worker is spawned (cold-loads from event store). |
| **`SNAPSHOT_FLUSH_INTERVAL`** | 100ms | Yes | Maximum time between snapshot flushes (time-based threshold). |
| **`SNAPSHOT_FLUSH_COUNT`** | 25 events | Yes | Maximum number of events processed between snapshot flushes (count-based threshold). The first threshold to be hit triggers the flush — whichever comes first. |
| **Worker channel capacity** | 256 | Yes, per aggregate | Per-worker command buffer size. Overflow → `try_send` failure → caller handles backpressure. |

### Serialization

| Component | Format | Rationale |
|-----------|--------|-----------|
| **Snapshot state** | protobuf | Easier migration across versions; schema evolution support |
| **Event data** | protobuf | Consistent with snapshots; single codec for the whole system |

### Worker loop (updated)

The worker loop now tracks both a **timer** and a **message counter** for snapshot flushing:

```rust
async fn run(&mut self) {
    let mut msg_counter = 0u64;
    let mut flush_timer = tokio::time::interval(self.config.snapshot_flush_interval);

    loop {
        tokio::select! {
            msg = self.cmd_rx.recv() => {
                match msg {
                    Some(Msg::Dispatch(cmd)) => {
                        msg_counter += 1;

                        // 1. Pure core: reduce command against in-memory state
                        let (new_state, events) = reduce(&self.state, cmd)?;

                        // 2. Commit: append events to store, publish
                        self.commit(events, &new_state).await?;

                        // 3. Update in-memory state
                        self.state = new_state;

                        // 4. Periodic snapshot flush (whichever comes first)
                        if msg_counter >= self.config.snapshot_flush_count {
                            self.flush_snapshot().await?;
                            msg_counter = 0;
                        }
                    }
                    Some(Msg::FlushSnapshot) => {
                        self.flush_snapshot().await?;
                    }
                    Some(Msg::Shutdown) | None => {
                        self.flush_snapshot().await?;
                        break;
                    }
                }
            }
            _ = flush_timer.tick() => {
                // Time-based threshold hit — flush if we have pending events
                if msg_counter > 0 {
                    self.flush_snapshot().await?;
                    msg_counter = 0;
                }
            }
            _ = &mut self.idle_timer => {
                // Idle timeout — flush snapshot and drop
                self.flush_snapshot().await?;
                break;
            }
        }
    }
}
```

Note: The `flush_timer` ticks continuously and triggers a flush only if there are un-flushed events (`msg_counter > 0`). This avoids redundant flushes when there's no activity.