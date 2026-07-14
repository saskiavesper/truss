use async_trait::async_trait;
use chrono::{DateTime, Utc};

// ============================================================================
// Shared IO Error
// ============================================================================

/// Errors that can occur during IO operations (database, event bus, etc.)
#[derive(Debug, thiserror::Error)]
pub enum IOError {
    #[error("database error: {0}")]
    Database(String),

    #[error("event bus error: {0}")]
    EventBus(String),
}

// ============================================================================
// Generic Committed Event
// ============================================================================

/// A committed event with a timestamp.
/// Generic over the domain-specific event kind (e.g. `organization::EventKind`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<T> {
    pub kind: T,
    pub timestamp: DateTime<Utc>,
}

impl<T> Event<T> {
    /// Create a new committed event with the current timestamp
    pub fn new(kind: T) -> Self {
        Self {
            kind,
            timestamp: Utc::now(),
        }
    }
}

// ============================================================================
// Shared Event Bus Capability
// ============================================================================

/// Capability to publish events to a message bus.
///
/// Generic over the committed event type `E` so each domain provides its own
/// concrete `Event<EventKind>`.
#[async_trait]
pub trait HasEventBus<E> {
    async fn publish_events(&self, events: &[E]) -> Result<(), IOError>;
}
