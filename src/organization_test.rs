use crate::event::{HasEventBus, IOError};
use crate::organization::*;
use chrono::Utc;

// ============================================================================
// Value Struct Tests
// ============================================================================

#[test]
fn organization_id_new_generates_unique_ids() {
    let id1 = OrganizationId::new();
    let id2 = OrganizationId::new();
    assert_ne!(id1, id2);
}

#[test]
fn organization_id_as_uuid() {
    let id = OrganizationId::new();
    let uuid = id.as_uuid();
    assert_eq!(&id.into_uuid(), uuid);
}

#[test]
fn organization_id_display() {
    let id = OrganizationId::new();
    assert_eq!(id.to_string(), id.as_uuid().to_string());
}

#[test]
fn organization_code_as_str() {
    let code = OrganizationCode::new("ACME-2024");
    assert_eq!(code.as_str(), "ACME-2024");
}

#[test]
fn crm_reference_as_str() {
    let reference = CrmReference::new("SF-001234");
    assert_eq!(reference.as_str(), "SF-001234");
}

#[test]
fn organization_name_as_str() {
    let name = OrganizationName::new("Acme Corp");
    assert_eq!(name.as_str(), "Acme Corp");
}

// ============================================================================
// Validation Tests (Pure)
// ============================================================================

#[test]
fn validate_name_success() {
    let name = OrganizationName::new("Acme Corp");
    assert!(validate_name(&name).is_ok());
}

#[test]
fn validate_name_empty_fails() {
    let name = OrganizationName::new("");
    let result = validate_name(&name);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NameValidationError::Empty));
}

#[test]
fn validate_name_too_long_fails() {
    let name = OrganizationName::new("a".repeat(256));
    let result = validate_name(&name);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NameValidationError::TooLong));
}

#[test]
fn validate_name_max_length_succeeds() {
    let name = OrganizationName::new("a".repeat(255));
    assert!(validate_name(&name).is_ok());
}

#[test]
fn validate_crm_reference_success() {
    let reference = CrmReference::new("SF-001");
    assert!(validate_crm_reference(&reference).is_ok());
}

#[test]
fn validate_crm_reference_empty_fails() {
    let reference = CrmReference::new("");
    let result = validate_crm_reference(&reference);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CrmReferenceValidationError::Empty
    ));
}

// ============================================================================
// Create Organization (Pure Function Tests)
// ============================================================================

#[test]
fn create_organization_success() {
    let id = OrganizationId::new();
    let name = OrganizationName::new("Acme Corp");
    let description = Some("A great company".to_string());
    let cmd = CreateOrganization {
        id,
        name: name.clone(),
        description: description.clone(),
    };

    let result = create_organization(cmd);

    assert!(result.is_ok());
    let (state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(state.id, id);
    assert_eq!(state.name, name);
    assert_eq!(state.description, description);

    match &events[0] {
        UncommittedEvent::Created {
            id: event_id,
            name: event_name,
            description: event_desc,
        } => {
            assert_eq!(*event_id, id);
            assert_eq!(*event_name, name);
            assert_eq!(*event_desc, description);
        }
        _ => panic!("Expected Created event"),
    }
}

#[test]
fn create_organization_with_no_description() {
    let cmd = CreateOrganization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Acme Corp"),
        description: None,
    };

    let result = create_organization(cmd);
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
fn create_organization_empty_name_fails() {
    let cmd = CreateOrganization {
        id: OrganizationId::new(),
        name: OrganizationName::new(""),
        description: None,
    };

    let result = create_organization(cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateError::InvalidName(NameValidationError::Empty)
    ));
}

#[test]
fn create_organization_long_name_fails() {
    let cmd = CreateOrganization {
        id: OrganizationId::new(),
        name: OrganizationName::new("a".repeat(256)),
        description: None,
    };

    let result = create_organization(cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateError::InvalidName(NameValidationError::TooLong)
    ));
}

// ============================================================================
// Update Organization (Pure Function Tests)
// ============================================================================

#[test]
fn update_organization_success() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Old Name"),
        description: None,
        code: None,
        crm_reference: None,
    };
    let cmd = UpdateOrganization {
        id: existing.id,
        name: OrganizationName::new("Updated Corp"),
        description: Some("New description".to_string()),
    };

    let result = update_organization(&existing, cmd.clone());

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(new_state.name.as_str(), "Updated Corp");
    assert_eq!(new_state.description.as_deref(), Some("New description"));

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
fn update_organization_clear_description() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Old Name"),
        description: Some("Old desc".to_string()),
        code: None,
        crm_reference: None,
    };
    let cmd = UpdateOrganization {
        id: existing.id,
        name: OrganizationName::new("Updated Corp"),
        description: None,
    };

    let result = update_organization(&existing, cmd);
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
fn update_organization_empty_name_fails() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Old Name"),
        description: None,
        code: None,
        crm_reference: None,
    };
    let cmd = UpdateOrganization {
        id: existing.id,
        name: OrganizationName::new(""),
        description: None,
    };

    let result = update_organization(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::InvalidName(NameValidationError::Empty)
    ));
}

// ============================================================================
// Generate Code (Pure Function Tests)
// ============================================================================

#[test]
fn generate_organization_code_success() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Acme"),
        description: None,
        code: None,
        crm_reference: None,
    };
    let cmd = GenerateOrganizationCode { id: existing.id };

    let result = generate_organization_code(&existing, cmd);

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert!(new_state.code.is_some());
    assert_eq!(new_state.code.as_ref().unwrap().as_str().len(), 8);

    match &events[0] {
        UncommittedEvent::CodeGenerated {
            id: event_id, code, ..
        } => {
            assert_eq!(*event_id, existing.id);
            assert_eq!(code.as_str().len(), 8);
            assert_eq!(code.as_str(), &code.as_str().to_uppercase());
        }
        _ => panic!("Expected CodeGenerated event"),
    }
}

#[test]
fn generate_organization_code_already_exists_fails() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Acme"),
        description: None,
        code: Some(OrganizationCode::new("EXISTING")),
        crm_reference: None,
    };
    let cmd = GenerateOrganizationCode { id: existing.id };

    let result = generate_organization_code(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GenerateCodeError::CodeAlreadyExists
    ));
}

// ============================================================================
// Attach CRM Reference (Pure Function Tests)
// ============================================================================

#[test]
fn attach_crm_reference_success() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Acme"),
        description: None,
        code: None,
        crm_reference: None,
    };
    let cmd = AttachCrmReference {
        id: existing.id,
        reference: CrmReference::new("SF-001234"),
    };

    let result = attach_crm_reference(&existing, cmd.clone());

    assert!(result.is_ok());
    let (new_state, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(new_state.crm_reference.as_ref().unwrap().as_str(), "SF-001234");

    match &events[0] {
        UncommittedEvent::CrmReferenceAttached {
            id: event_id,
            reference: event_ref,
            ..
        } => {
            assert_eq!(*event_id, existing.id);
            assert_eq!(*event_ref, cmd.reference);
        }
        _ => panic!("Expected CrmReferenceAttached event"),
    }
}

#[test]
fn attach_crm_reference_empty_fails() {
    let existing = Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Acme"),
        description: None,
        code: None,
        crm_reference: None,
    };
    let cmd = AttachCrmReference {
        id: existing.id,
        reference: CrmReference::new(""),
    };

    let result = attach_crm_reference(&existing, cmd);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AttachCrmReferenceError::InvalidCrmReference(CrmReferenceValidationError::Empty)
    ));
}

// ============================================================================
// Event Commit Tests
// ============================================================================

#[test]
fn event_commit_adds_timestamp() {
    let uncommitted = UncommittedEvent::Created {
        id: OrganizationId::new(),
        name: OrganizationName::new("Test"),
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

fn test_organization() -> Organization {
    Organization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Test Org"),
        description: None,
        code: None,
        crm_reference: None,
    }
}

#[test]
fn apply_event_created() {
    let mut org = test_organization();

    let id = OrganizationId::new();
    let event = Event {
        kind: EventKind::Created {
            id,
            name: OrganizationName::new("Acme Corp"),
            description: Some("A company".to_string()),
        },
        timestamp: Utc::now(),
    };

    apply_event(&mut org, &event.kind);

    assert_eq!(org.id, id);
    assert_eq!(org.name.as_str(), "Acme Corp");
    assert_eq!(org.description.as_deref(), Some("A company"));
    assert!(org.code.is_none());
    assert!(org.crm_reference.is_none());
}

#[test]
fn apply_event_updated() {
    let mut org = test_organization();

    let event = Event {
        kind: EventKind::Updated {
            id: org.id,
            name: OrganizationName::new("New Name"),
            description: Some("New desc".to_string()),
        },
        timestamp: Utc::now(),
    };

    apply_event(&mut org, &event.kind);

    assert_eq!(org.name.as_str(), "New Name");
    assert_eq!(org.description.as_deref(), Some("New desc"));
}

#[test]
fn apply_event_code_generated() {
    let mut org = test_organization();

    let event = Event {
        kind: EventKind::CodeGenerated {
            id: org.id,
            code: OrganizationCode::new("ACME-1234"),
        },
        timestamp: Utc::now(),
    };

    apply_event(&mut org, &event.kind);

    assert_eq!(org.code.unwrap().as_str(), "ACME-1234");
}

#[test]
fn apply_event_crm_reference_attached() {
    let mut org = test_organization();

    let event = Event {
        kind: EventKind::CrmReferenceAttached {
            id: org.id,
            reference: CrmReference::new("SF-001"),
        },
        timestamp: Utc::now(),
    };

    apply_event(&mut org, &event.kind);

    assert_eq!(org.crm_reference.unwrap().as_str(), "SF-001");
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn error_display_messages() {
    assert_eq!(
        NameValidationError::Empty.to_string(),
        "organization name cannot be empty"
    );
    assert!(
        NameValidationError::TooLong
            .to_string()
            .contains("organization name too long")
    );

    assert_eq!(
        GenerateCodeError::CodeAlreadyExists.to_string(),
        "organization already has a code assigned"
    );
    assert_eq!(
        CrmReferenceValidationError::Empty.to_string(),
        "CRM reference cannot be empty"
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
    organizations: Mutex<HashMap<OrganizationId, Organization>>,
    published_events: Mutex<Vec<Event>>,
}

impl MockContext {
    fn new() -> Self {
        Self {
            organizations: Mutex::new(HashMap::new()),
            published_events: Mutex::new(Vec::new()),
        }
    }

    fn add_organization(&self, org: Organization) {
        self.organizations.lock().unwrap().insert(org.id, org);
    }

    fn published_events(&self) -> Vec<Event> {
        self.published_events.lock().unwrap().clone()
    }
}

// Implement the capability traits separately

#[async_trait]
impl HasOrganizationReader for MockContext {
    async fn load_organization(&self, id: OrganizationId) -> Result<Option<Organization>, IOError> {
        Ok(self.organizations.lock().unwrap().get(&id).cloned())
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
    let cmd = CreateOrganization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Acme Corp"),
        description: Some("A great company".to_string()),
    };

    let result = handle_create(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);

    match &ctx.published_events()[0].kind {
        EventKind::Created { name, .. } => {
            assert_eq!(name.as_str(), "Acme Corp");
        }
        _ => panic!("Expected Created event"),
    }
}

#[tokio::test]
async fn handle_create_invalid_name_fails() {
    let ctx = MockContext::new();
    let cmd = CreateOrganization {
        id: OrganizationId::new(),
        name: OrganizationName::new(""),
        description: None,
    };

    let result = handle_create(&ctx, cmd).await;

    assert!(result.is_err());
    assert!(ctx.published_events().is_empty());
}

#[tokio::test]
async fn handle_update_success() {
    let ctx = MockContext::new();
    let id = OrganizationId::new();
    ctx.add_organization(Organization {
        id,
        name: OrganizationName::new("Old Name"),
        description: None,
        code: None,
        crm_reference: None,
    });

    let cmd = UpdateOrganization {
        id,
        name: OrganizationName::new("Updated Corp"),
        description: None,
    };

    let result = handle_update(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_update_not_found_fails() {
    let ctx = MockContext::new();
    let cmd = UpdateOrganization {
        id: OrganizationId::new(),
        name: OrganizationName::new("Doesn't Exist"),
        description: None,
    };

    let result = handle_update(&ctx, cmd).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn handle_generate_code_success() {
    let ctx = MockContext::new();
    let id = OrganizationId::new();
    ctx.add_organization(Organization {
        id,
        name: OrganizationName::new("Acme"),
        description: None,
        code: None,
        crm_reference: None,
    });

    let cmd = GenerateOrganizationCode { id };

    let result = handle_generate_code(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_generate_code_already_exists_fails() {
    let ctx = MockContext::new();
    let id = OrganizationId::new();
    ctx.add_organization(Organization {
        id,
        name: OrganizationName::new("Acme"),
        description: None,
        code: Some(OrganizationCode::new("EXISTING")),
        crm_reference: None,
    });

    let cmd = GenerateOrganizationCode { id };

    let result = handle_generate_code(&ctx, cmd).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GenerateCodeError::CodeAlreadyExists
    ));
}

#[tokio::test]
async fn handle_attach_crm_reference_success() {
    let ctx = MockContext::new();
    let id = OrganizationId::new();
    ctx.add_organization(Organization {
        id,
        name: OrganizationName::new("Acme"),
        description: None,
        code: None,
        crm_reference: None,
    });

    let cmd = AttachCrmReference {
        id,
        reference: CrmReference::new("SF-001"),
    };

    let result = handle_attach_crm_reference(&ctx, cmd).await;

    assert!(result.is_ok());
    assert_eq!(ctx.published_events().len(), 1);
}

#[tokio::test]
async fn handle_attach_crm_reference_not_found_fails() {
    let ctx = MockContext::new();
    let cmd = AttachCrmReference {
        id: OrganizationId::new(),
        reference: CrmReference::new("SF-001"),
    };

    let result = handle_attach_crm_reference(&ctx, cmd).await;

    assert!(result.is_err());
}
