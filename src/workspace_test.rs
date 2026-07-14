use crate::event::{HasEventBus, IOError};
use crate::organization::OrganizationId;
use crate::workspace::*;
use chrono::Utc;
use uuid::Uuid;

// ============================================================================
// Value Struct Tests
// ============================================================================

#[test]
fn workspace_id_new_generates_unique_ids() {
    let id1 = WorkspaceId::new();
    let id2 = WorkspaceId::new();
    assert_ne!(id1, id2);
}

#[test]
fn workspace_id_as_uuid() {
    let id = WorkspaceId::new();
    let uuid = id.as_uuid();
    assert_eq!(&id.into_uuid(), uuid);
}

#[test]
fn workspace_id_display() {
    let id = WorkspaceId::new();
    assert_eq!(id.to_string(), id.as_uuid().to_string());
}

#[test]
fn workspace_name_as_str() {
    let name = WorkspaceName::new("My Project");
    assert_eq!(name.as_str(), "My Project");
}

#[test]
fn member_ref_new() {
    let uuid = Uuid::new_v4();
    let member = MemberRef::new(uuid);
    assert_eq!(member.as_uuid(), &uuid);
}

#[test]
fn member_ref_from_uuid() {
    let uuid = Uuid::new_v4();
    let member: MemberRef = uuid.into();
    assert_eq!(member.into_uuid(), uuid);
}

#[test]
fn task_ref_new() {
    let uuid = Uuid::new_v4();
    let task = TaskRef::new(uuid);
    assert_eq!(task.as_uuid(), &uuid);
}

#[test]
fn task_ref_from_uuid() {
    let uuid = Uuid::new_v4();
    let task: TaskRef = uuid.into();
    assert_eq!(task.into_uuid(), uuid);
}

#[test]
fn member_ref_equality() {
    let uuid = Uuid::new_v4();
    let a = MemberRef::new(uuid);
    let b = MemberRef::new(uuid);
    assert_eq!(a, b);
}

#[test]
fn task_ref_equality() {
    let uuid = Uuid::new_v4();
    let a = TaskRef::new(uuid);
    let b = TaskRef::new(uuid);
    assert_eq!(a, b);
}

// ============================================================================
// Validation Tests (Pure)
// ============================================================================

#[test]
fn validate_name_success() {
    let name = WorkspaceName::new("My Workspace");
    assert!(validate_name(&name).is_ok());
}

#[test]
fn validate_name_empty_fails() {
    let name = WorkspaceName::new("");
    let result = validate_name(&name);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NameValidationError::Empty));
}

#[test]
fn validate_name_too_long_fails() {
    let name = WorkspaceName::new("a".repeat(256));
    let result = validate_name(&name);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NameValidationError::TooLong));
}

#[test]
fn validate_name_max_length_succeeds() {
    let name = WorkspaceName::new("a".repeat(255));
    assert!(validate_name(&name).is_ok());
}

// ============================================================================
// Create Workspace (Pure Function Tests)
// ============================================================================

#[test]
fn create_workspace_success() {
    let id = WorkspaceId::new();
    let org_id = OrganizationId::new();
    let name = WorkspaceName::new("Sprint Board");
    let description = Some("Track our sprint progress".to_string());
    let cmd = CreateWorkspace {
        id,
        organization_id: org_id,
        name: name.clone(),
        description: description.clone(),
    };

    let result = create_workspace(cmd);

    assert!(result.is_ok());
    let (state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(state.id, id);
    assert_eq!(state.organization_id, org_id);
    assert_eq!(state.name, name);
    assert_eq!(state.description, description);
    assert!(state.members.is_empty());
    assert!(state.tasks.is_empty());
    assert!(state.created_at.is_none());
    assert!(state.updated_at.is_none());

    match &events[0] {
        UncommittedEvent::Created {
            id: event_id,
            organization_id: event_org_id,
            name: event_name,
            description: event_desc,
        } => {
            assert_eq!(*event_id, id);
            assert_eq!(*event_org_id, org_id);
            assert_eq!(*event_name, name);
            assert_eq!(*event_desc, description);
        }
        _ => panic!("Expected Created event"),
    }
}

#[test]
fn create_workspace_with_no_description() {
    let cmd = CreateWorkspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Sprint Board"),
        description: None,
    };

    let result = create_workspace(cmd);
    assert!(result.is_ok());

    let (state, events) = result.unwrap();
    assert!(state.description.is_none());

    match &events[0] {
        UncommittedEvent::Created { description, .. } => {
            assert!(description.is_none());
        }
        _ => panic!("Expected Created event"),
    }
}

#[test]
fn create_workspace_empty_name_fails() {
    let cmd = CreateWorkspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new(""),
        description: None,
    };

    let result = create_workspace(cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateError::InvalidName(NameValidationError::Empty)
    ));
}

#[test]
fn create_workspace_long_name_fails() {
    let cmd = CreateWorkspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("a".repeat(256)),
        description: None,
    };

    let result = create_workspace(cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateError::InvalidName(NameValidationError::TooLong)
    ));
}

// ============================================================================
// Update Workspace (Pure Function Tests)
// ============================================================================

fn test_workspace() -> Workspace {
    Workspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Test Workspace"),
        description: None,
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    }
}

#[test]
fn update_workspace_success() {
    let existing = test_workspace();
    let cmd = UpdateWorkspace {
        id: existing.id,
        name: WorkspaceName::new("Updated Board"),
        description: Some("New description".to_string()),
    };

    let result = update_workspace(&existing, cmd.clone());

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(new_state.name.as_str(), "Updated Board");
    assert_eq!(new_state.description.as_deref(), Some("New description"));
    assert_eq!(new_state.organization_id, existing.organization_id);
    assert!(new_state.members.is_empty());
    assert!(new_state.tasks.is_empty());

    match &events[0] {
        UncommittedEvent::Updated {
            id: event_id,
            name: event_name,
            description: event_desc,
        } => {
            assert_eq!(*event_id, existing.id);
            assert_eq!(*event_name, cmd.name);
            assert_eq!(*event_desc, cmd.description);
        }
        _ => panic!("Expected Updated event"),
    }
}

#[test]
fn update_workspace_clear_description() {
    let existing = Workspace {
        description: Some("Old desc".to_string()),
        ..test_workspace()
    };
    let cmd = UpdateWorkspace {
        id: existing.id,
        name: WorkspaceName::new("Updated Board"),
        description: None,
    };

    let result = update_workspace(&existing, cmd);
    assert!(result.is_ok());

    let (new_state, events) = result.unwrap();
    assert!(new_state.description.is_none());

    match &events[0] {
        UncommittedEvent::Updated { description, .. } => {
            assert!(description.is_none());
        }
        _ => panic!("Expected Updated event"),
    }
}

#[test]
fn update_workspace_empty_name_fails() {
    let existing = test_workspace();
    let cmd = UpdateWorkspace {
        id: existing.id,
        name: WorkspaceName::new(""),
        description: None,
    };

    let result = update_workspace(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::InvalidName(NameValidationError::Empty)
    ));
}

// ============================================================================
// Add / Remove Member (Pure Function Tests)
// ============================================================================

#[test]
fn add_member_success() {
    let existing = test_workspace();
    let member = MemberRef::new(Uuid::new_v4());
    let cmd = AddMember {
        workspace_id: existing.id,
        member,
    };

    let result = add_member(&existing, cmd);

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(new_state.members.len(), 1);
    assert_eq!(new_state.members[0], member);

    match &events[0] {
        UncommittedEvent::MemberAdded {
            workspace_id,
            member: event_member,
        } => {
            assert_eq!(*workspace_id, existing.id);
            assert_eq!(*event_member, member);
        }
        _ => panic!("Expected MemberAdded event"),
    }
}

#[test]
fn add_member_already_exists_fails() {
    let member = MemberRef::new(Uuid::new_v4());
    let existing = Workspace {
        members: vec![member],
        ..test_workspace()
    };
    let cmd = AddMember {
        workspace_id: existing.id,
        member,
    };

    let result = add_member(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MemberError::AlreadyExists));
}

#[test]
fn add_multiple_members() {
    let existing = test_workspace();
    let member_a = MemberRef::new(Uuid::new_v4());
    let member_b = MemberRef::new(Uuid::new_v4());

    let (state_a, _) = add_member(
        &existing,
        AddMember {
            workspace_id: existing.id,
            member: member_a,
        },
    )
    .unwrap();

    let (state_b, _) = add_member(
        &state_a,
        AddMember {
            workspace_id: existing.id,
            member: member_b,
        },
    )
    .unwrap();

    assert_eq!(state_b.members.len(), 2);
    assert!(state_b.members.contains(&member_a));
    assert!(state_b.members.contains(&member_b));
}

#[test]
fn remove_member_success() {
    let member = MemberRef::new(Uuid::new_v4());
    let existing = Workspace {
        members: vec![member],
        ..test_workspace()
    };
    let cmd = RemoveMember {
        workspace_id: existing.id,
        member,
    };

    let result = remove_member(&existing, cmd);

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert!(new_state.members.is_empty());

    match &events[0] {
        UncommittedEvent::MemberRemoved {
            workspace_id,
            member: event_member,
        } => {
            assert_eq!(*workspace_id, existing.id);
            assert_eq!(*event_member, member);
        }
        _ => panic!("Expected MemberRemoved event"),
    }
}

#[test]
fn remove_member_not_found_fails() {
    let existing = test_workspace();
    let cmd = RemoveMember {
        workspace_id: existing.id,
        member: MemberRef::new(Uuid::new_v4()),
    };

    let result = remove_member(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MemberError::NotFound));
}

#[test]
fn remove_member_preserves_other_members() {
    let member_a = MemberRef::new(Uuid::new_v4());
    let member_b = MemberRef::new(Uuid::new_v4());
    let existing = Workspace {
        members: vec![member_a, member_b],
        ..test_workspace()
    };
    let cmd = RemoveMember {
        workspace_id: existing.id,
        member: member_a,
    };

    let (new_state, _) = remove_member(&existing, cmd).unwrap();

    assert_eq!(new_state.members.len(), 1);
    assert_eq!(new_state.members[0], member_b);
}

// ============================================================================
// Add / Remove Task (Pure Function Tests)
// ============================================================================

#[test]
fn add_task_success() {
    let existing = test_workspace();
    let task = TaskRef::new(Uuid::new_v4());
    let cmd = AddTask {
        workspace_id: existing.id,
        task,
    };

    let result = add_task(&existing, cmd);

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(new_state.tasks.len(), 1);
    assert_eq!(new_state.tasks[0], task);

    match &events[0] {
        UncommittedEvent::TaskAdded {
            workspace_id,
            task: event_task,
        } => {
            assert_eq!(*workspace_id, existing.id);
            assert_eq!(*event_task, task);
        }
        _ => panic!("Expected TaskAdded event"),
    }
}

#[test]
fn add_task_already_exists_fails() {
    let task = TaskRef::new(Uuid::new_v4());
    let existing = Workspace {
        tasks: vec![task],
        ..test_workspace()
    };
    let cmd = AddTask {
        workspace_id: existing.id,
        task,
    };

    let result = add_task(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TaskError::AlreadyExists));
}

#[test]
fn add_multiple_tasks() {
    let existing = test_workspace();
    let task_a = TaskRef::new(Uuid::new_v4());
    let task_b = TaskRef::new(Uuid::new_v4());

    let (state_a, _) = add_task(
        &existing,
        AddTask {
            workspace_id: existing.id,
            task: task_a,
        },
    )
    .unwrap();

    let (state_b, _) = add_task(
        &state_a,
        AddTask {
            workspace_id: existing.id,
            task: task_b,
        },
    )
    .unwrap();

    assert_eq!(state_b.tasks.len(), 2);
    assert!(state_b.tasks.contains(&task_a));
    assert!(state_b.tasks.contains(&task_b));
}

#[test]
fn remove_task_success() {
    let task = TaskRef::new(Uuid::new_v4());
    let existing = Workspace {
        tasks: vec![task],
        ..test_workspace()
    };
    let cmd = RemoveTask {
        workspace_id: existing.id,
        task,
    };

    let result = remove_task(&existing, cmd);

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert!(new_state.tasks.is_empty());

    match &events[0] {
        UncommittedEvent::TaskRemoved {
            workspace_id,
            task: event_task,
        } => {
            assert_eq!(*workspace_id, existing.id);
            assert_eq!(*event_task, task);
        }
        _ => panic!("Expected TaskRemoved event"),
    }
}

#[test]
fn remove_task_not_found_fails() {
    let existing = test_workspace();
    let cmd = RemoveTask {
        workspace_id: existing.id,
        task: TaskRef::new(Uuid::new_v4()),
    };

    let result = remove_task(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TaskError::NotFound));
}

#[test]
fn remove_task_preserves_other_tasks() {
    let task_a = TaskRef::new(Uuid::new_v4());
    let task_b = TaskRef::new(Uuid::new_v4());
    let existing = Workspace {
        tasks: vec![task_a, task_b],
        ..test_workspace()
    };
    let cmd = RemoveTask {
        workspace_id: existing.id,
        task: task_a,
    };

    let (new_state, _) = remove_task(&existing, cmd).unwrap();

    assert_eq!(new_state.tasks.len(), 1);
    assert_eq!(new_state.tasks[0], task_b);
}

// ============================================================================
// Event Commit Tests
// ============================================================================

#[test]
fn event_commit_adds_timestamp() {
    let uncommitted = UncommittedEvent::Created {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Test"),
        description: None,
    };

    let before = Utc::now();
    let committed = Event::new(EventKind::from(uncommitted));
    let after = Utc::now();

    assert!(committed.timestamp >= before);
    assert!(committed.timestamp <= after);
}

// ============================================================================
// Apply Event Tests (Pure)
// ============================================================================

#[test]
fn apply_event_created() {
    let mut ws = test_workspace();
    let ts = Utc::now();

    let id = WorkspaceId::new();
    let org_id = OrganizationId::new();
    let event = Event {
        kind: EventKind::Created {
            id,
            organization_id: org_id,
            name: WorkspaceName::new("Sprint Board"),
            description: Some("Track progress".to_string()),
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert_eq!(ws.id, id);
    assert_eq!(ws.organization_id, org_id);
    assert_eq!(ws.name.as_str(), "Sprint Board");
    assert_eq!(ws.description.as_deref(), Some("Track progress"));
    assert!(ws.members.is_empty());
    assert!(ws.tasks.is_empty());
    assert_eq!(ws.created_at, Some(ts));
    assert_eq!(ws.updated_at, Some(ts));
}

#[test]
fn apply_event_updated() {
    let mut ws = test_workspace();
    let ts = Utc::now();

    let event = Event {
        kind: EventKind::Updated {
            id: ws.id,
            name: WorkspaceName::new("New Name"),
            description: Some("New desc".to_string()),
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert_eq!(ws.name.as_str(), "New Name");
    assert_eq!(ws.description.as_deref(), Some("New desc"));
    assert_eq!(ws.updated_at, Some(ts));
}

#[test]
fn apply_event_member_added() {
    let mut ws = test_workspace();
    let ts = Utc::now();
    let member = MemberRef::new(Uuid::new_v4());

    let event = Event {
        kind: EventKind::MemberAdded {
            workspace_id: ws.id,
            member,
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert_eq!(ws.members.len(), 1);
    assert_eq!(ws.members[0], member);
    assert_eq!(ws.updated_at, Some(ts));
}

#[test]
fn apply_event_member_removed() {
    let member = MemberRef::new(Uuid::new_v4());
    let mut ws = Workspace {
        members: vec![member],
        ..test_workspace()
    };
    let ts = Utc::now();

    let event = Event {
        kind: EventKind::MemberRemoved {
            workspace_id: ws.id,
            member,
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert!(ws.members.is_empty());
    assert_eq!(ws.updated_at, Some(ts));
}

#[test]
fn apply_event_task_added() {
    let mut ws = test_workspace();
    let ts = Utc::now();
    let task = TaskRef::new(Uuid::new_v4());

    let event = Event {
        kind: EventKind::TaskAdded {
            workspace_id: ws.id,
            task,
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert_eq!(ws.tasks.len(), 1);
    assert_eq!(ws.tasks[0], task);
    assert_eq!(ws.updated_at, Some(ts));
}

#[test]
fn apply_event_task_removed() {
    let task = TaskRef::new(Uuid::new_v4());
    let mut ws = Workspace {
        tasks: vec![task],
        ..test_workspace()
    };
    let ts = Utc::now();

    let event = Event {
        kind: EventKind::TaskRemoved {
            workspace_id: ws.id,
            task,
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert!(ws.tasks.is_empty());
    assert_eq!(ws.updated_at, Some(ts));
}

#[test]
fn apply_event_created_preserves_existing_state() {
    let mut ws = Workspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Old"),
        description: Some("Old desc".to_string()),
        members: vec![MemberRef::new(Uuid::new_v4())],
        tasks: vec![TaskRef::new(Uuid::new_v4())],
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };
    let ts = Utc::now();
    let new_id = WorkspaceId::new();
    let new_org_id = OrganizationId::new();

    let event = Event {
        kind: EventKind::Created {
            id: new_id,
            organization_id: new_org_id,
            name: WorkspaceName::new("New Board"),
            description: None,
        },
        timestamp: ts,
    };

    apply_event(&mut ws, &event.kind, &event.timestamp);

    assert_eq!(ws.id, new_id);
    assert_eq!(ws.organization_id, new_org_id);
    assert_eq!(ws.name.as_str(), "New Board");
    assert!(ws.description.is_none());
    assert!(ws.members.is_empty());
    assert!(ws.tasks.is_empty());
    assert_eq!(ws.created_at, Some(ts));
    assert_eq!(ws.updated_at, Some(ts));
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn error_display_messages() {
    assert_eq!(
        NameValidationError::Empty.to_string(),
        "workspace name cannot be empty"
    );
    assert!(
        NameValidationError::TooLong
            .to_string()
            .contains("workspace name too long")
    );

    assert_eq!(
        MemberError::AlreadyExists.to_string(),
        "member already exists in workspace"
    );
    assert_eq!(
        MemberError::NotFound.to_string(),
        "member not found in workspace"
    );
    assert_eq!(
        TaskError::AlreadyExists.to_string(),
        "task already exists in workspace"
    );
    assert_eq!(
        TaskError::NotFound.to_string(),
        "task not found in workspace"
    );
}

// ============================================================================
// Handler Tests with Mock Context
// ============================================================================

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

/// Mock context for testing handlers
struct MockContext {
    workspaces: Mutex<HashMap<WorkspaceId, Workspace>>,
    published_events: Mutex<Vec<Event>>,
}

impl MockContext {
    fn new() -> Self {
        Self {
            workspaces: Mutex::new(HashMap::new()),
            published_events: Mutex::new(Vec::new()),
        }
    }

    fn add_workspace(&self, ws: Workspace) {
        self.workspaces.lock().unwrap().insert(ws.id, ws);
    }

    fn published_events(&self) -> Vec<Event> {
        self.published_events.lock().unwrap().clone()
    }
}

#[async_trait]
impl HasWorkspaceReader for MockContext {
    async fn load_workspace(&self, id: WorkspaceId) -> Result<Option<Workspace>, IOError> {
        Ok(self.workspaces.lock().unwrap().get(&id).cloned())
    }
}

#[async_trait]
impl HasEventBus<Event> for MockContext {
    async fn publish_events(&self, events: &[Event]) -> Result<(), IOError> {
        self.published_events
            .lock()
            .unwrap()
            .extend_from_slice(events);
        Ok(())
    }
}

#[tokio::test]
async fn handle_create_success() {
    let ctx = MockContext::new();
    let cmd = CreateWorkspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Sprint Board"),
        description: Some("Track our sprints".to_string()),
    };

    let result = handle_create(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);

    match &ctx.published_events()[0].kind {
        EventKind::Created { name, .. } => {
            assert_eq!(name.as_str(), "Sprint Board");
        }
        _ => panic!("Expected Created event"),
    }

    // Verify committed event has a timestamp
    assert!(
        ctx.published_events()[0].timestamp <= Utc::now(),
        "event timestamp should be in the past or present"
    );
}

#[tokio::test]
async fn handle_create_invalid_name_fails() {
    let ctx = MockContext::new();
    let cmd = CreateWorkspace {
        id: WorkspaceId::new(),
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new(""),
        description: None,
    };

    let result = handle_create(&ctx, cmd).await;

    assert!(result.is_err());
    assert!(ctx.published_events().is_empty());
}

#[tokio::test]
async fn handle_update_success() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Old Name"),
        description: None,
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    });

    let cmd = UpdateWorkspace {
        id,
        name: WorkspaceName::new("Updated Board"),
        description: None,
    };

    let result = handle_update(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_update_not_found_fails() {
    let ctx = MockContext::new();
    let cmd = UpdateWorkspace {
        id: WorkspaceId::new(),
        name: WorkspaceName::new("Doesn't Exist"),
        description: None,
    };

    let result = handle_update(&ctx, cmd).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn handle_add_member_success() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Board"),
        description: None,
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    });

    let cmd = AddMember {
        workspace_id: id,
        member: MemberRef::new(Uuid::new_v4()),
    };

    let result = handle_add_member(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_add_member_not_found_fails() {
    let ctx = MockContext::new();
    let cmd = AddMember {
        workspace_id: WorkspaceId::new(),
        member: MemberRef::new(Uuid::new_v4()),
    };

    let result = handle_add_member(&ctx, cmd).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn handle_remove_member_success() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    let member = MemberRef::new(Uuid::new_v4());
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Board"),
        description: None,
        members: vec![member],
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    });

    let cmd = RemoveMember {
        workspace_id: id,
        member,
    };

    let result = handle_remove_member(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_remove_member_not_found_fails() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Board"),
        description: None,
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    });

    let cmd = RemoveMember {
        workspace_id: id,
        member: MemberRef::new(Uuid::new_v4()),
    };

    let result = handle_remove_member(&ctx, cmd).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn handle_add_task_success() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Board"),
        description: None,
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    });

    let cmd = AddTask {
        workspace_id: id,
        task: TaskRef::new(Uuid::new_v4()),
    };

    let result = handle_add_task(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_add_task_not_found_fails() {
    let ctx = MockContext::new();
    let cmd = AddTask {
        workspace_id: WorkspaceId::new(),
        task: TaskRef::new(Uuid::new_v4()),
    };

    let result = handle_add_task(&ctx, cmd).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn handle_remove_task_success() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    let task = TaskRef::new(Uuid::new_v4());
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Board"),
        description: None,
        members: Vec::new(),
        tasks: vec![task],
        created_at: None,
        updated_at: None,
    });

    let cmd = RemoveTask {
        workspace_id: id,
        task,
    };

    let result = handle_remove_task(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_remove_task_not_found_fails() {
    let ctx = MockContext::new();
    let id = WorkspaceId::new();
    ctx.add_workspace(Workspace {
        id,
        organization_id: OrganizationId::new(),
        name: WorkspaceName::new("Board"),
        description: None,
        members: Vec::new(),
        tasks: Vec::new(),
        created_at: None,
        updated_at: None,
    });

    let cmd = RemoveTask {
        workspace_id: id,
        task: TaskRef::new(Uuid::new_v4()),
    };

    let result = handle_remove_task(&ctx, cmd).await;

    assert!(result.is_err());
}
