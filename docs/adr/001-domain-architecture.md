# ADR-001: Domain Architecture Pattern

## Status

Accepted

## Context

Truss is a B2B project management and orchestration platform that will handle complex business workflows, task orchestration, and multi-organization collaboration. We need an architecture that:

1. Scales with domain complexity
2. Is testable without heavy mocking
3. Enforces type safety and clear separation of concerns
4. Supports event-driven architecture (NATS)
5. Follows Rust idioms and project conventions (value structs, monomorphization)

## Decision

We adopt a **Functional Core / Imperative Shell** architecture with **Capability Traits** for dependency injection.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  IMPERATIVE SHELL                                               │
│  - Context trait (Has*) for IO dependencies                     │
│  - Handler functions (orchestration)                            │
│  - Event commit (add timestamps)                                │
│  - State persistence                                            │
│  - Event publishing                                             │
└───────────────────────────┬─────────────────────────────────────┘
                            │ calls
┌───────────────────────────▼─────────────────────────────────────┐
│  FUNCTIONAL CORE                                                │
│  - Value structs (OrganizationId, OrganizationName, etc.)       │
│  - Pure functions (reducer pattern)                             │
│  - Validation logic                                             │
│  - State transitions                                            │
│  - Event production                                             │
└─────────────────────────────────────────────────────────────────┘
```

### Key Patterns

#### 1. Value Structs (Newtypes)

Domain values are wrapped in newtype structs for type safety:

```rust
pub struct OrganizationId(Uuid);
pub struct OrganizationName(String);
pub struct OrganizationCode(String);
pub struct CrmReference(String);
```

**Benefits:**
- Prevents mixing up IDs (OrganizationId vs UserId)
- Encapsulates validation and formatting
- Inner fields are private with controlled access

#### 2. Reduce & Apply Patterns (Pure Functions)

Each domain defines **two** pure functions: `reduce` for command handling and `apply_event` for replay.

**`reduce`** — validates a command against current state and returns new state + events:

```rust
pub fn reduce_organization(
    state: &Organization,
    cmd: UpdateOrganization,
) -> Result<(Organization, Vec<UncommittedEvent>), UpdateError> {
    // Validate
    validate_name(&cmd.name)?;
    
    // Build new state
    let new_state = Organization {
        name: cmd.name.clone(),
        description: cmd.description.clone(),
        ..state.clone()
    };
    
    // Build events (no timestamp — shell adds that at commit time)
    let events = vec![UncommittedEvent::Updated {
        id: cmd.id,
        name: cmd.name,
        description: cmd.description,
    }];
    
    Ok((new_state, events))
}
```

**`apply_event`** — folds a single event into state (used to rebuild state from the event stream):

```rust
pub fn apply_organization_event(state: &Organization, event: &OrganizationEvent) -> Organization {
    match event {
        OrganizationEvent::Created(e) => Organization {
            id: e.id,
            name: e.name.clone(),
            ..Default::default()
        },
        OrganizationEvent::Updated(e) => Organization {
            name: e.name.clone(),
            description: e.description.clone(),
            ..state.clone()
        },
    }
}
```

**Benefits:**
- Pure functions: no IO, no side effects
- Easy to test without mocks
- State transitions are explicit and auditable
- Replay is just `events.iter().fold(init, apply_event)` — no branching logic needed

#### 3. Capability Traits (Has Pattern)

IO dependencies are injected via capability traits. The naming reflects what the trait provides to the handler:

```rust
#[async_trait]
pub trait HasOrganizationState {
    /// Load the current state of an aggregate by reading the latest snapshot
    /// and replaying any events since that snapshot. Handlers never touch events directly.
    async fn load_state(&self, id: OrganizationId) -> Result<Option<Organization>, IoError>;
}

#[async_trait]
pub trait HasEventCommitter {
    /// Convert uncommitted events → committed (add timestamps, checksums, versions),
    /// append to store, update snapshot, and publish to event bus.
    async fn commit_events(&self, id: AggregateId, events: Vec<UncommittedEvent>, expected_version: Version) -> Result<(), IoError>;
}
```

**Implementation detail (hidden behind the trait):**

```rust
impl HasOrganizationState for RocksDbConnection {
    async fn load_state(&self, id: OrganizationId) -> Result<Option<Organization>, IoError> {
        // 1. Read latest snapshot from snapshots column family (fast path)
        let (snapshot, snapshot_version) = self.read_snapshot::<Organization>("organization", id).await?;

        // 2. Read events since snapshot version (if any)
        let events = self.read_events_from("organization", id, snapshot_version).await?;

        // 3. Fold new events onto snapshot state
        let state = events.iter()
            .map(|e| deserialize::<OrganizationEvent>(&e.data))
            .fold(snapshot.unwrap_or_default(), |s, e| apply_organization_event(&s, &e));

        Ok(Some(state))
    }
}
```

**Benefits:**
- Handlers request only the state they need — no knowledge of events or snapshots
- Easy to mock for testing (mock returns a pre-built `Organization`)
- Different implementations for different environments (snapshot-only for tests, snapshot+events for prod)
- Composable: `C: HasOrganizationState + HasEventCommitter`

#### 4. Event Types

Events are split into uncommitted (from core) and committed (stored with metadata):

```rust
// From pure core - no timestamp, no version, no checksum
pub enum UncommittedEvent {
    Created { id: OrganizationId, name: OrganizationName },
    Updated { id: OrganizationId, name: OrganizationName, description: Option<String> },
}

// Committed by shell - has all infrastructure fields
pub struct StoredEvent {
    pub global_position: Position,
    pub stream_id: AggregateId,
    pub version: Version,
    pub event_type: String,
    pub data: Vec<u8>,              // serialized UncommittedEvent
    pub metadata: EventMetadata,    // timestamp, causation/correlation ids, actor
    pub prev_checksum: String,      // SHA-256 of previous event ("0" for first)
    pub checksum: String,           // SHA-256(prev || type || data || timestamp)
}
```

**Benefits:**
- Core remains pure (no `Utc::now()`, no checksum computation)
- Infrastructure fields added at commit time by the shell
- Checksum chain makes the event store itself a tamper-evident ledger

#### 5. Event Commit Helper

The shell provides a shared `commit_events` utility used by all handlers. It converts uncommitted → committed, assigns versions, computes checksums, appends to the store, and publishes:

```rust
/// Shared helper — every handler calls this instead of writing its own commit logic.
pub async fn commit_events<C: HasEventStore + HasEventBus>(
    ctx: &C,
    aggregate_id: AggregateId,
    uncommitted: Vec<UncommittedEvent>,
    expected_version: Version,
    causation_id: EventId,
    correlation_id: CorrelationId,
    actor_id: UserId,
) -> Result<(), IoError> {
    let now = Utc::now();
    let mut stored: Vec<StoredEvent> = uncommitted.into_iter().enumerate().map(|(i, ev)| {
        let version = Version(expected_version.0 + i as u64 + 1);
        let event_type = ev.event_type();
        let data = serialize(&ev);
        let prev = /* load previous checksum from store, or "0" */;
        StoredEvent {
            version,
            event_type,
            data,
            metadata: EventMetadata {
                timestamp: now,
                causation_id,
                correlation_id,
                actor_id,
            },
            prev_checksum: prev,
            checksum: sha256!(prev || event_type || data || now),
            ..Default::default()  // global_position assigned by store on append
        }
    }).collect();

    ctx.append_events(aggregate_id, &mut stored, expected_version).await?;
    ctx.publish(&stored).await?;
    Ok(())
}
```

**Benefits:**
- Single place for versioning, checksumming, and timestamping logic
- Handlers stay thin — orchestrate, don't encode plumbing

#### 6. Error Handling

Errors are split by layer:

```rust
// Domain errors (from pure core)
pub enum UpdateError {
    InvalidName(NameValidationError),
}

// IO errors (from handlers)
pub enum IoError {
    Database(String),
    EventBus(String),
}

// Combined in handler results
pub enum UpdateError {
    InvalidName(NameValidationError),  // from core
    Io(IoError),                        // from context
}
```

### File Structure

Each domain follows this structure:

```
src/
├── organization.rs           # Domain types, pure functions, handlers
├── organization_test.rs      # Tests
├── user.rs                   # Another domain...
└── user_test.rs
```

### Handler Pattern

Handlers orchestrate the full load → fold → reduce → commit flow:

```rust
pub async fn handle_update<C>(ctx: &C, cmd: UpdateOrganization) -> Result<(), UpdateError>
where
    C: HasOrganizationState + HasEventCommitter,
{
    // 1. Load current state (imperative — snapshot + events, handler doesn't care)
    let state = ctx.load_state(cmd.id).await.map_err(UpdateError::Io)?
        .ok_or(UpdateError::NotFound)?;

    // 2. Pure core: validate command against state, produce new state + events
    let (_new_state, uncommitted) = reduce_organization(&state, cmd).map_err(UpdateError::Domain)?;

    // 3. Commit: shell adds timestamps, checksums, version, updates snapshot — then append + publish
    commit_events(ctx, cmd.id.into(), uncommitted, state.version, /* causation/correlation/actor */).await?;

    Ok(())
}
```

**Key constraint:** Step 2 is pure — it runs on borrowed data and produces new values. All side effects (loading, committing, publishing) are confined to steps 1 and 3.

**What `commit_events` does beyond event persistence:** after appending events, it also **updates the snapshot** in the snapshots column family so the next `load_state` starts from the latest state.

## Consequences

### Positive

1. **Testability**: Pure functions can be tested without any mocking
2. **Type Safety**: Value structs prevent mixing up domain types
3. **Scalability**: Architecture grows with domain complexity
4. **Clarity**: Clear separation between pure logic and IO
5. **Flexibility**: Capability traits allow different implementations per environment

### Negative

1. **Boilerplate**: More types and traits than a simple CRUD approach
2. **Learning Curve**: Team needs to understand functional core / imperative shell
3. **Initial Setup**: More upfront work for simple domains

### Mitigations

- The boilerplate pays off as domain complexity grows
- Document patterns clearly (this ADR)
- Consider simpler patterns for truly simple domains

## Alternatives Considered

1. **Simple CRUD with Service Layer**: Rejected - doesn't scale with complexity
2. **Full CQRS with Separate Read/Write Models**: Overkill for initial implementation (separate read/write database clusters add deployment and consistency complexity). **Lightweight in-process read-model projectors** (same RocksDB instance, different column families) ARE in scope from Phase 0 — they subscribe to the event bus and fold events into query-optimized views. This is not the same as full separate-database CQRS; it's a pragmatic middle ground that gives us query flexibility without the operational overhead.
3. **Actor Model**: Considered but adds unnecessary complexity

## References

- Functional Core, Imperative Shell (Gary Bernhardt)
- Hexagonal Architecture (Alistair Cockburn)
- Event Sourcing patterns
