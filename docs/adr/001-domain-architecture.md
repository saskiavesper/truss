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

#### 2. Reducer Pattern (Pure Functions)

State transitions are pure functions that take current state and return new state + events:

```rust
pub fn update_organization(
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
    
    // Build events
    let events = vec![UncommittedEvent::Updated {
        id: cmd.id,
        name: cmd.name,
        description: cmd.description,
    }];
    
    Ok((new_state, events))
}
```

**Benefits:**
- Pure functions: no IO, no side effects
- Easy to test without mocks
- State transitions are explicit and auditable

#### 3. Capability Traits (Has Pattern)

IO dependencies are injected via capability traits:

```rust
#[async_trait]
pub trait HasOrganizationReader {
    async fn load_organization(&self, id: OrganizationId) -> Result<Option<Organization>, IoError>;
}

#[async_trait]
pub trait HasEventBus {
    async fn publish_events(&self, events: &[Event]) -> Result<(), IoError>;
}
```

**Benefits:**
- Handlers request only the capabilities they need
- Easy to mock for testing
- Different implementations for different environments
- Composable: `C: HasOrganizationReader + HasEventBus`

#### 4. Event Types

Events are split into uncommitted (from core) and committed (with timestamps):

```rust
// From pure core - no timestamp
pub enum UncommittedEvent {
    Created { id, name, description },
    Updated { id, name, description },
    // ...
}

// Committed by shell - has timestamp
pub struct Event {
    pub kind: EventKind,
    pub timestamp: DateTime<Utc>,
}
```

**Benefits:**
- Core remains pure (no `Utc::now()`)
- Timestamps added at commit time
- Clear separation of concerns

#### 5. Error Handling

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

Handlers orchestrate the flow:

```rust
pub async fn handle_update<C>(ctx: &C, cmd: UpdateOrganization) -> Result<(), UpdateError>
where
    C: HasOrganizationReader + HasEventBus,
{
    // 1. Load state (imperative)
    let state = require_organization(ctx, cmd.id).await?;
    
    // 2. Pure core: validate + produce (new_state, events)
    let (_new_state, uncommitted) = update_organization(&state, cmd)?;
    
    // 3. Commit events: add timestamps + publish
    commit_and_publish(ctx, uncommitted).await?;
    
    Ok(())
}
```

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
2. **Full CQRS with Separate Read/Write Models**: Overkill for initial implementation
3. **Actor Model**: Considered but adds unnecessary complexity

## References

- Functional Core, Imperative Shell (Gary Bernhardt)
- Hexagonal Architecture (Alistair Cockburn)
- Event Sourcing patterns
