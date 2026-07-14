use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::event::{self, IOError, HasEventBus};
use crate::organization::OrganizationId;

// ============================================================================
// Value Structs - Domain Types
// ============================================================================

/// Unique identifier for a workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(Uuid);

impl WorkspaceId {
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

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Workspace display name
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceName(String);

impl WorkspaceName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorkspaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reference to a workspace member (user)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemberRef(Uuid);

impl MemberRef {
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self(id.into())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for MemberRef {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for MemberRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reference to a workspace task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskRef(Uuid);

impl TaskRef {
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self(id.into())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for TaskRef {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for TaskRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Validation (Pure Functions)
// ============================================================================

const MAX_NAME_LENGTH: usize = 255;

/// Validate workspace name
pub fn validate_name(name: &WorkspaceName) -> Result<(), NameValidationError> {
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
    #[error("workspace name cannot be empty")]
    Empty,

    #[error("workspace name too long (max {MAX_NAME_LENGTH} characters)")]
    TooLong,
}

// ============================================================================
// Commands (Input Data)
// ============================================================================

/// Command to create a new workspace
#[derive(Debug, Clone)]
pub struct CreateWorkspace {
    pub id: WorkspaceId,
    pub organization_id: OrganizationId,
    pub name: WorkspaceName,
    pub description: Option<String>,
}

/// Command to update an existing workspace
#[derive(Debug, Clone)]
pub struct UpdateWorkspace {
    pub id: WorkspaceId,
    pub name: WorkspaceName,
    pub description: Option<String>,
}

/// Command to add a member to a workspace
#[derive(Debug, Clone)]
pub struct AddMember {
    pub workspace_id: WorkspaceId,
    pub member: MemberRef,
}

/// Command to remove a member from a workspace
#[derive(Debug, Clone)]
pub struct RemoveMember {
    pub workspace_id: WorkspaceId,
    pub member: MemberRef,
}

/// Command to add a task reference to a workspace
#[derive(Debug, Clone)]
pub struct AddTask {
    pub workspace_id: WorkspaceId,
    pub task: TaskRef,
}

/// Command to remove a task reference from a workspace
#[derive(Debug, Clone)]
pub struct RemoveTask {
    pub workspace_id: WorkspaceId,
    pub task: TaskRef,
}

// ============================================================================
// Events (Uncommitted - without timestamps)
// ============================================================================

/// Uncommitted event - produced by pure functions, timestamps added by shell
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UncommittedEvent {
    Created {
        id: WorkspaceId,
        organization_id: OrganizationId,
        name: WorkspaceName,
        description: Option<String>,
    },
    Updated {
        id: WorkspaceId,
        name: WorkspaceName,
        description: Option<String>,
    },
    MemberAdded {
        workspace_id: WorkspaceId,
        member: MemberRef,
    },
    MemberRemoved {
        workspace_id: WorkspaceId,
        member: MemberRef,
    },
    TaskAdded {
        workspace_id: WorkspaceId,
        task: TaskRef,
    },
    TaskRemoved {
        workspace_id: WorkspaceId,
        task: TaskRef,
    },
}

/// Committed event - with timestamp, ready for persistence
pub type Event = event::Event<EventKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Created {
        id: WorkspaceId,
        organization_id: OrganizationId,
        name: WorkspaceName,
        description: Option<String>,
    },
    Updated {
        id: WorkspaceId,
        name: WorkspaceName,
        description: Option<String>,
    },
    MemberAdded {
        workspace_id: WorkspaceId,
        member: MemberRef,
    },
    MemberRemoved {
        workspace_id: WorkspaceId,
        member: MemberRef,
    },
    TaskAdded {
        workspace_id: WorkspaceId,
        task: TaskRef,
    },
    TaskRemoved {
        workspace_id: WorkspaceId,
        task: TaskRef,
    },
}

impl From<UncommittedEvent> for EventKind {
    fn from(uncommitted: UncommittedEvent) -> Self {
        match uncommitted {
            UncommittedEvent::Created {
                id,
                organization_id,
                name,
                description,
            } => EventKind::Created {
                id,
                organization_id,
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
            UncommittedEvent::MemberAdded {
                workspace_id,
                member,
            } => EventKind::MemberAdded {
                workspace_id,
                member,
            },
            UncommittedEvent::MemberRemoved {
                workspace_id,
                member,
            } => EventKind::MemberRemoved {
                workspace_id,
                member,
            },
            UncommittedEvent::TaskAdded {
                workspace_id,
                task,
            } => EventKind::TaskAdded {
                workspace_id,
                task,
            },
            UncommittedEvent::TaskRemoved {
                workspace_id,
                task,
            } => EventKind::TaskRemoved {
                workspace_id,
                task,
            },
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

/// Errors that can occur during member operations
#[derive(Debug, thiserror::Error)]
pub enum MemberError {
    #[error("member already exists in workspace")]
    AlreadyExists,

    #[error("member not found in workspace")]
    NotFound,

    #[error(transparent)]
    Io(#[from] IOError),
}

/// Errors that can occur during task operations
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("task already exists in workspace")]
    AlreadyExists,

    #[error("task not found in workspace")]
    NotFound,

    #[error(transparent)]
    Io(#[from] IOError),
}

// ============================================================================
// Capability Traits (Has Pattern)
// ============================================================================

/// Capability to read workspace state
#[async_trait]
pub trait HasWorkspaceReader {
    async fn load_workspace(&self, id: WorkspaceId) -> Result<Option<Workspace>, IOError>;
}


// ============================================================================
// Aggregate State
// ============================================================================

/// Workspace aggregate state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub organization_id: OrganizationId,
    pub name: WorkspaceName,
    pub description: Option<String>,
    pub members: Vec<MemberRef>,
    pub tasks: Vec<TaskRef>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Functional Core - Pure Functions (Reducer Pattern)
// ============================================================================

/// Pure function: Create workspace
/// Returns new state and events to emit
pub fn create_workspace(
    cmd: CreateWorkspace,
) -> Result<(Workspace, Vec<UncommittedEvent>), CreateError> {
    validate_name(&cmd.name)?;

    let state = Workspace {
        id: cmd.id,
        organization_id: cmd.organization_id,
        name: cmd.name.clone(),
        description: cmd.description.clone(),
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    };

    let events = vec![UncommittedEvent::Created {
        id: cmd.id,
        organization_id: cmd.organization_id,
        name: cmd.name,
        description: cmd.description,
    }];

    Ok((state, events))
}

/// Pure function: Update workspace
/// Takes current state, returns new state and events
pub fn update_workspace(
    state: &Workspace,
    cmd: UpdateWorkspace,
) -> Result<(Workspace, Vec<UncommittedEvent>), UpdateError> {
    validate_name(&cmd.name)?;

    let new_state = Workspace {
        id: state.id,
        organization_id: state.organization_id,
        name: cmd.name.clone(),
        description: cmd.description.clone(),
        members: state.members.clone(),
        tasks: state.tasks.clone(),
        created_at: state.created_at,
        updated_at: state.updated_at,
    };

    let events = vec![UncommittedEvent::Updated {
        id: cmd.id,
        name: cmd.name,
        description: cmd.description,
    }];

    Ok((new_state, events))
}

/// Pure function: Add member to workspace
/// Takes current state, returns new state and events
pub fn add_member(
    state: &Workspace,
    cmd: AddMember,
) -> Result<(Workspace, Vec<UncommittedEvent>), MemberError> {
    if state.members.contains(&cmd.member) {
        return Err(MemberError::AlreadyExists);
    }

    let mut new_members = state.members.clone();
    new_members.push(cmd.member);

    let new_state = Workspace {
        members: new_members,
        ..state.clone()
    };

    let events = vec![UncommittedEvent::MemberAdded {
        workspace_id: cmd.workspace_id,
        member: cmd.member,
    }];

    Ok((new_state, events))
}

/// Pure function: Remove member from workspace
/// Takes current state, returns new state and events
pub fn remove_member(
    state: &Workspace,
    cmd: RemoveMember,
) -> Result<(Workspace, Vec<UncommittedEvent>), MemberError> {
    let pos = state
        .members
        .iter()
        .position(|m| *m == cmd.member)
        .ok_or(MemberError::NotFound)?;

    let mut new_members = state.members.clone();
    new_members.remove(pos);

    let new_state = Workspace {
        members: new_members,
        ..state.clone()
    };

    let events = vec![UncommittedEvent::MemberRemoved {
        workspace_id: cmd.workspace_id,
        member: cmd.member,
    }];

    Ok((new_state, events))
}

/// Pure function: Add task reference to workspace
/// Takes current state, returns new state and events
pub fn add_task(
    state: &Workspace,
    cmd: AddTask,
) -> Result<(Workspace, Vec<UncommittedEvent>), TaskError> {
    if state.tasks.contains(&cmd.task) {
        return Err(TaskError::AlreadyExists);
    }

    let mut new_tasks = state.tasks.clone();
    new_tasks.push(cmd.task);

    let new_state = Workspace {
        tasks: new_tasks,
        ..state.clone()
    };

    let events = vec![UncommittedEvent::TaskAdded {
        workspace_id: cmd.workspace_id,
        task: cmd.task,
    }];

    Ok((new_state, events))
}

/// Pure function: Remove task reference from workspace
/// Takes current state, returns new state and events
pub fn remove_task(
    state: &Workspace,
    cmd: RemoveTask,
) -> Result<(Workspace, Vec<UncommittedEvent>), TaskError> {
    let pos = state
        .tasks
        .iter()
        .position(|t| *t == cmd.task)
        .ok_or(TaskError::NotFound)?;

    let mut new_tasks = state.tasks.clone();
    new_tasks.remove(pos);

    let new_state = Workspace {
        tasks: new_tasks,
        ..state.clone()
    };

    let events = vec![UncommittedEvent::TaskRemoved {
        workspace_id: cmd.workspace_id,
        task: cmd.task,
    }];

    Ok((new_state, events))
}

/// Pure function: Apply event to workspace state
/// Takes the event timestamp to set created_at / updated_at
pub fn apply_event(state: &mut Workspace, event: &EventKind, timestamp: &DateTime<Utc>) {
    match event {
        EventKind::Created {
            id,
            organization_id,
            name,
            description,
        } => {
            state.id = *id;
            state.organization_id = *organization_id;
            state.name = name.clone();
            state.description = description.clone();
            state.members = Vec::new();
            state.tasks = Vec::new();
            state.created_at = Some(*timestamp);
            state.updated_at = Some(*timestamp);
        }
        EventKind::Updated {
            name, description, ..
        } => {
            state.name = name.clone();
            state.description = description.clone();
            state.updated_at = Some(*timestamp);
        }
        EventKind::MemberAdded { member, .. } => {
            if !state.members.contains(member) {
                state.members.push(*member);
            }
            state.updated_at = Some(*timestamp);
        }
        EventKind::MemberRemoved { member, .. } => {
            state.members.retain(|m| m != member);
            state.updated_at = Some(*timestamp);
        }
        EventKind::TaskAdded { task, .. } => {
            if !state.tasks.contains(task) {
                state.tasks.push(*task);
            }
            state.updated_at = Some(*timestamp);
        }
        EventKind::TaskRemoved { task, .. } => {
            state.tasks.retain(|t| t != task);
            state.updated_at = Some(*timestamp);
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

/// Helper to load workspace or return NotFound
async fn require_workspace<C: HasWorkspaceReader>(
    ctx: &C,
    id: WorkspaceId,
) -> Result<Workspace, IOError> {
    ctx.load_workspace(id)
        .await?
        .ok_or(IOError::Database("workspace not found".to_string()))
}

/// Handle create workspace command
/// Requires: EventBus (no read needed - creates new state)
pub async fn handle_create<C>(ctx: &C, cmd: CreateWorkspace) -> Result<(), CreateError>
where
    C: HasEventBus<Event>,
{
    let (mut new_state, uncommitted) = create_workspace(cmd)?;
    let committed = commit_and_publish(ctx, uncommitted).await?;

    // Set timestamps from the committed event
    if let Some(event) = committed.first() {
        new_state.created_at = Some(event.timestamp);
        new_state.updated_at = Some(event.timestamp);
    }

    // TODO: persist new_state
    let _ = new_state;
    Ok(())
}

/// Handle update workspace command
/// Requires: WorkspaceReader + EventBus
pub async fn handle_update<C>(ctx: &C, cmd: UpdateWorkspace) -> Result<(), UpdateError>
where
    C: HasWorkspaceReader + HasEventBus<Event>,
{
    let state = require_workspace(ctx, cmd.id).await?;
    let (mut new_state, uncommitted) = update_workspace(&state, cmd)?;
    let committed = commit_and_publish(ctx, uncommitted).await?;

    // Set updated_at from the committed event
    if let Some(event) = committed.first() {
        new_state.updated_at = Some(event.timestamp);
    }

    // TODO: persist new_state
    let _ = new_state;
    Ok(())
}

/// Handle add member command
/// Requires: WorkspaceReader + EventBus
pub async fn handle_add_member<C>(ctx: &C, cmd: AddMember) -> Result<(), MemberError>
where
    C: HasWorkspaceReader + HasEventBus<Event>,
{
    let state = require_workspace(ctx, cmd.workspace_id).await?;
    let (mut new_state, uncommitted) = add_member(&state, cmd)?;
    let committed = commit_and_publish(ctx, uncommitted).await?;

    // Set updated_at from the committed event
    if let Some(event) = committed.first() {
        new_state.updated_at = Some(event.timestamp);
    }

    // TODO: persist new_state
    let _ = new_state;
    Ok(())
}

/// Handle remove member command
/// Requires: WorkspaceReader + EventBus
pub async fn handle_remove_member<C>(ctx: &C, cmd: RemoveMember) -> Result<(), MemberError>
where
    C: HasWorkspaceReader + HasEventBus<Event>,
{
    let state = require_workspace(ctx, cmd.workspace_id).await?;
    let (mut new_state, uncommitted) = remove_member(&state, cmd)?;
    let committed = commit_and_publish(ctx, uncommitted).await?;

    // Set updated_at from the committed event
    if let Some(event) = committed.first() {
        new_state.updated_at = Some(event.timestamp);
    }

    // TODO: persist new_state
    let _ = new_state;
    Ok(())
}

/// Handle add task command
/// Requires: WorkspaceReader + EventBus
pub async fn handle_add_task<C>(ctx: &C, cmd: AddTask) -> Result<(), TaskError>
where
    C: HasWorkspaceReader + HasEventBus<Event>,
{
    let state = require_workspace(ctx, cmd.workspace_id).await?;
    let (mut new_state, uncommitted) = add_task(&state, cmd)?;
    let committed = commit_and_publish(ctx, uncommitted).await?;

    // Set updated_at from the committed event
    if let Some(event) = committed.first() {
        new_state.updated_at = Some(event.timestamp);
    }

    // TODO: persist new_state
    let _ = new_state;
    Ok(())
}

/// Handle remove task command
/// Requires: WorkspaceReader + EventBus
pub async fn handle_remove_task<C>(ctx: &C, cmd: RemoveTask) -> Result<(), TaskError>
where
    C: HasWorkspaceReader + HasEventBus<Event>,
{
    let state = require_workspace(ctx, cmd.workspace_id).await?;
    let (mut new_state, uncommitted) = remove_task(&state, cmd)?;
    let committed = commit_and_publish(ctx, uncommitted).await?;

    // Set updated_at from the committed event
    if let Some(event) = committed.first() {
        new_state.updated_at = Some(event.timestamp);
    }

    // TODO: persist new_state
    let _ = new_state;
    Ok(())
}
