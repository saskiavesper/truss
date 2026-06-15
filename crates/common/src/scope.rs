use crate::domain::Id;
use std::ops::Deref;

/// Represents the authenticated or anonymous user context within a session.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Actor {
    /// An unauthenticated user with no identity or permissions.
    Anonymous,
    /// An authenticated user with a unique [`Id`], optional tenant, and a list of permission strings.
    Authenticated { id: Id, tenant: Option<Id>, permissions: Vec<String> },
}

impl Actor {
    /// Returns `true` if the actor is authenticated.
    #[must_use]
    pub const fn is_authenticated(&self) -> bool {
        matches!(self, Self::Authenticated { .. })
    }

    /// Returns `true` if the actor is anonymous.
    #[must_use]
    pub const fn is_anonymous(&self) -> bool {
        matches!(self, Self::Anonymous)
    }
}

/// Wraps an [`Actor`] and enforces invariants at construction time.
///
/// Currently, requires that authenticated actors have at least one permission.
/// Internal types does the other validations
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scope {
    actor: Actor,
}

/// Errors returned by [`Scope::new`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScopeError {
    /// The authenticated actor has an empty permission list.
    InvalidPermissions,
}

impl Scope {
    /// Creates a new [`Scope`], validating the provided [`Actor`].
    ///
    /// # Errors
    ///
    /// Returns [`ScopeError::InvalidPermissions`] when `actor` is
    /// [`Actor::Authenticated`] with an empty `permissions` list.
    pub fn new(actor: Actor) -> Result<Self, ScopeError> {
        if let Actor::Authenticated { permissions, .. } = &actor
            && permissions.is_empty()
        {
            return Err(ScopeError::InvalidPermissions);
        }
        Ok(Self { actor })
    }
}

/// Allows `&Scope` to dereference to `&Actor`, providing access to `Actor` methods directly.
impl Deref for Scope {
    type Target = Actor;

    fn deref(&self) -> &Self::Target {
        &self.actor
    }
}
