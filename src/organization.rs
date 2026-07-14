use async_trait::async_trait;
use uuid::Uuid;

use crate::event::{self, IOError, HasEventBus};

// ============================================================================
// Value Structs - Domain Types
// ============================================================================

/// Unique identifier for an organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrganizationId(Uuid);

impl OrganizationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for OrganizationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OrganizationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Organization invite/referral code (e.g., "ACME-2024")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrganizationCode(String);

impl OrganizationCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OrganizationCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// External CRM reference identifier (e.g., Salesforce Account ID)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrmReference(String);

impl CrmReference {
    pub fn new(reference: impl Into<String>) -> Self {
        Self(reference.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CrmReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Organization display name
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationName(String);

impl OrganizationName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OrganizationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Validation (Pure Functions)
// ============================================================================

const MAX_NAME_LENGTH: usize = 255;

/// Validate organization name
pub fn validate_name(name: &OrganizationName) -> Result<(), NameValidationError> {
    if name.as_str().is_empty() {
        return Err(NameValidationError::Empty);
    }
    if name.as_str().len() > MAX_NAME_LENGTH {
        return Err(NameValidationError::TooLong);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum NameValidationError {
    #[error("organization name cannot be empty")]
    Empty,

    #[error("organization name too long (max {MAX_NAME_LENGTH} characters)")]
    TooLong,
}

/// Validate CRM reference
pub fn validate_crm_reference(reference: &CrmReference) -> Result<(), CrmReferenceValidationError> {
    if reference.as_str().is_empty() {
        return Err(CrmReferenceValidationError::Empty);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CrmReferenceValidationError {
    #[error("CRM reference cannot be empty")]
    Empty,
}

// ============================================================================
// Commands (Input Data)
// ============================================================================

/// Command to create a new organization
#[derive(Debug, Clone)]
pub struct CreateOrganization {
    pub id: OrganizationId,
    pub name: OrganizationName,
    pub description: Option<String>,
}

/// Command to update an existing organization
#[derive(Debug, Clone)]
pub struct UpdateOrganization {
    pub id: OrganizationId,
    pub name: OrganizationName,
    pub description: Option<String>,
}

/// Command to generate a code for an organization
#[derive(Debug, Clone)]
pub struct GenerateOrganizationCode {
    pub id: OrganizationId,
}

/// Command to attach a CRM reference to an organization
#[derive(Debug, Clone)]
pub struct AttachCrmReference {
    pub id: OrganizationId,
    pub reference: CrmReference,
}

// ============================================================================
// Events (Uncommitted - without timestamps)
// ============================================================================

/// Uncommitted event - produced by pure functions, timestamps added by shell
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UncommittedEvent {
    Created {
        id: OrganizationId,
        name: OrganizationName,
        description: Option<String>,
    },
    Updated {
        id: OrganizationId,
        name: OrganizationName,
        description: Option<String>,
    },
    CodeGenerated {
        id: OrganizationId,
        code: OrganizationCode,
    },
    CrmReferenceAttached {
        id: OrganizationId,
        reference: CrmReference,
    },
}

/// Committed event - with timestamp, ready for persistence
pub type Event = event::Event<EventKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Created {
        id: OrganizationId,
        name: OrganizationName,
        description: Option<String>,
    },
    Updated {
        id: OrganizationId,
        name: OrganizationName,
        description: Option<String>,
    },
    CodeGenerated {
        id: OrganizationId,
        code: OrganizationCode,
    },
    CrmReferenceAttached {
        id: OrganizationId,
        reference: CrmReference,
    },
}

impl From<UncommittedEvent> for EventKind {
    fn from(uncommitted: UncommittedEvent) -> Self {
        match uncommitted {
            UncommittedEvent::Created {
                id,
                name,
                description,
            } => EventKind::Created {
                id,
                name,
                description,
            },
            UncommittedEvent::Updated {
                id,
                name,
                description,
            } => EventKind::Updated {
                id,
                name,
                description,
            },
            UncommittedEvent::CodeGenerated { id, code } => EventKind::CodeGenerated { id, code },
            UncommittedEvent::CrmReferenceAttached { id, reference } => {
                EventKind::CrmReferenceAttached { id, reference }
            }
        }
    }
}

// ============================================================================
// Error Types
// ============================================================================



/// Errors that can occur during create operation
#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    InvalidName(#[from] NameValidationError),

    #[error(transparent)]
    Io(#[from] IOError),
}

/// Errors that can occur during update operation
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error(transparent)]
    InvalidName(#[from] NameValidationError),

    #[error(transparent)]
    Io(#[from] IOError),
}

/// Errors that can occur during generate code operation
#[derive(Debug, thiserror::Error)]
pub enum GenerateCodeError {
    #[error("organization already has a code assigned")]
    CodeAlreadyExists,

    #[error(transparent)]
    Io(#[from] IOError),
}

/// Errors that can occur during attach CRM reference operation
#[derive(Debug, thiserror::Error)]
pub enum AttachCrmReferenceError {
    #[error(transparent)]
    InvalidCrmReference(#[from] CrmReferenceValidationError),

    #[error(transparent)]
    Io(#[from] IOError),
}

// ============================================================================
// Capability Traits (Has Pattern)
// ============================================================================

/// Capability to read organization state
#[async_trait]
pub trait HasOrganizationReader {
    async fn load_organization(&self, id: OrganizationId) -> Result<Option<Organization>, IOError>;
}




// ============================================================================
// Aggregate State
// ============================================================================

/// Organization aggregate state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: OrganizationName,
    pub description: Option<String>,
    pub code: Option<OrganizationCode>,
    pub crm_reference: Option<CrmReference>,
}

// ============================================================================
// Functional Core - Pure Functions (Reducer Pattern)
// ============================================================================

/// Pure function: Create organization
/// Returns new state and events to emit
pub fn create_organization(
    cmd: CreateOrganization,
) -> Result<(Organization, Vec<UncommittedEvent>), CreateError> {
    validate_name(&cmd.name)?;

    let state = Organization {
        id: cmd.id,
        name: cmd.name.clone(),
        description: cmd.description.clone(),
        code: None,
        crm_reference: None,
    };

    let events = vec![UncommittedEvent::Created {
        id: cmd.id,
        name: cmd.name,
        description: cmd.description,
    }];

    Ok((state, events))
}

/// Pure function: Update organization
/// Takes current state, returns new state and events
pub fn update_organization(
    state: &Organization,
    cmd: UpdateOrganization,
) -> Result<(Organization, Vec<UncommittedEvent>), UpdateError> {
    validate_name(&cmd.name)?;

    let new_state = Organization {
        id: state.id,
        name: cmd.name.clone(),
        description: cmd.description.clone(),
        code: state.code.clone(),
        crm_reference: state.crm_reference.clone(),
    };

    let events = vec![UncommittedEvent::Updated {
        id: cmd.id,
        name: cmd.name,
        description: cmd.description,
    }];

    Ok((new_state, events))
}

/// Pure function: Generate organization code
/// Takes current state, returns new state and events
pub fn generate_organization_code(
    state: &Organization,
    cmd: GenerateOrganizationCode,
) -> Result<(Organization, Vec<UncommittedEvent>), GenerateCodeError> {
    if state.code.is_some() {
        return Err(GenerateCodeError::CodeAlreadyExists);
    }

    let code = OrganizationCode::new(format!(
        "{}",
        cmd.id
            .as_uuid()
            .as_simple()
            .to_string()[..8]
            .to_uppercase()
    ));

    let new_state = Organization {
        code: Some(code.clone()),
        ..state.clone()
    };

    let events = vec![UncommittedEvent::CodeGenerated { id: cmd.id, code }];

    Ok((new_state, events))
}

/// Pure function: Attach CRM reference
/// Takes current state, returns new state and events
pub fn attach_crm_reference(
    state: &Organization,
    cmd: AttachCrmReference,
) -> Result<(Organization, Vec<UncommittedEvent>), AttachCrmReferenceError> {
    validate_crm_reference(&cmd.reference)?;

    let new_state = Organization {
        crm_reference: Some(cmd.reference.clone()),
        ..state.clone()
    };

    let events = vec![UncommittedEvent::CrmReferenceAttached {
        id: cmd.id,
        reference: cmd.reference,
    }];

    Ok((new_state, events))
}

/// Pure function: Apply event to organization state
pub fn apply_event(state: &mut Organization, event: &EventKind) {
    match event {
        EventKind::Created {
            id,
            name,
            description,
        } => {
            state.id = *id;
            state.name = name.clone();
            state.description = description.clone();
        }
        EventKind::Updated {
            name,
            description,
            ..
        } => {
            state.name = name.clone();
            state.description = description.clone();
        }
        EventKind::CodeGenerated { code, .. } => {
            state.code = Some(code.clone());
        }
        EventKind::CrmReferenceAttached { reference, .. } => {
            state.crm_reference = Some(reference.clone());
        }
    }
}

// ============================================================================
// Imperative Shell - Handlers with Capability Bounds
// ============================================================================

/// Helper: commit events and publish
async fn commit_and_publish<C>(
    ctx: &C,
    uncommitted: Vec<UncommittedEvent>,
) -> Result<Vec<Event>, IOError>
where
    C: HasEventBus<Event>,
{
    let committed: Vec<Event> = uncommitted
        .into_iter()
        .map(|u| Event::new(EventKind::from(u)))
        .collect();
    ctx.publish_events(&committed).await?;
    Ok(committed)
}

/// Helper to load organization or return NotFound
async fn require_organization<C: HasOrganizationReader>(
    ctx: &C,
    id: OrganizationId,
) -> Result<Organization, IOError> {
    ctx.load_organization(id)
        .await?
        .ok_or(IOError::Database("organization not found".to_string()))
}

/// Handle create organization command
/// Requires: EventBus (no read needed - creates new state)
pub async fn handle_create<C>(ctx: &C, cmd: CreateOrganization) -> Result<(), CreateError>
where
    C: HasEventBus<Event>,
{
    let (_new_state, uncommitted) = create_organization(cmd)?;
    let _committed = commit_and_publish(ctx, uncommitted).await?;
    // TODO: persist _new_state
    Ok(())
}

/// Handle update organization command
/// Requires: OrganizationReader + EventBus
pub async fn handle_update<C>(ctx: &C, cmd: UpdateOrganization) -> Result<(), UpdateError>
where
    C: HasOrganizationReader + HasEventBus<Event>,
{
    let state = require_organization(ctx, cmd.id).await?;
    let (_new_state, uncommitted) = update_organization(&state, cmd)?;
    let _committed = commit_and_publish(ctx, uncommitted).await?;
    // TODO: persist _new_state
    Ok(())
}

/// Handle generate code command
/// Requires: OrganizationReader + EventBus
pub async fn handle_generate_code<C>(
    ctx: &C,
    cmd: GenerateOrganizationCode,
) -> Result<(), GenerateCodeError>
where
    C: HasOrganizationReader + HasEventBus<Event>,
{
    let state = require_organization(ctx, cmd.id).await?;
    let (_new_state, uncommitted) = generate_organization_code(&state, cmd)?;
    let _committed = commit_and_publish(ctx, uncommitted).await?;
    // TODO: persist _new_state
    Ok(())
}

/// Handle attach CRM reference command
/// Requires: OrganizationReader + EventBus
pub async fn handle_attach_crm_reference<C>(
    ctx: &C,
    cmd: AttachCrmReference,
) -> Result<(), AttachCrmReferenceError>
where
    C: HasOrganizationReader + HasEventBus<Event>,
{
    let state = require_organization(ctx, cmd.id).await?;
    let (_new_state, uncommitted) = attach_crm_reference(&state, cmd)?;
    let _committed = commit_and_publish(ctx, uncommitted).await?;
    // TODO: persist _new_state
    Ok(())
}
