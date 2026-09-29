use async_trait::async_trait;
use clrinf_core::{
    dispatcher::{DispatchError, Request, RequestHandler},
    rules::{BusinessRule, RuleResult},
    RequestContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contact {
    pub id: String,
    pub tenant_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub company: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct CreateContactCommand {
    pub tenant_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub company: String,
}

impl Request for CreateContactCommand {
    type Response = Contact;
}

#[derive(Debug, Clone)]
pub struct GetContactQuery {
    pub tenant_id: String,
    pub contact_id: String,
}

impl Request for GetContactQuery {
    type Response = Option<Contact>;
}

#[derive(Clone, Default)]
pub struct InMemoryContactRepository {
    storage: Arc<RwLock<HashMap<String, Contact>>>,
}

impl InMemoryContactRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<Contact> {
        let store = self.storage.read().await;
        store.get(&format!("{tenant_id}:{id}")).cloned()
    }

    pub async fn get_by_email(&self, tenant_id: &str, email: &str) -> Option<Contact> {
        let store = self.storage.read().await;
        store
            .values()
            .find(|c| c.tenant_id == tenant_id && c.email.eq_ignore_ascii_case(email))
            .cloned()
    }

    pub async fn save(&self, contact: Contact) -> Contact {
        let mut store = self.storage.write().await;
        store.insert(format!("{}:{}", contact.tenant_id, contact.id), contact.clone());
        contact
    }
}

// ─── Business Rules ────────────────────────────────────────────────────────

pub struct ValidateContactEmailFormatRule;

#[async_trait]
impl BusinessRule<CreateContactCommand> for ValidateContactEmailFormatRule {
    async fn evaluate(
        &self,
        context: &CreateContactCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if !context.email.contains('@') || !context.email.contains('.') {
            return RuleResult::failed(
                "INVALID_EMAIL_FORMAT",
                format!("'{}' is not a valid email address.", context.email),
            );
        }
        RuleResult::success()
    }
}

pub struct EnsureContactEmailUniqueRule {
    pub repo: InMemoryContactRepository,
}

#[async_trait]
impl BusinessRule<CreateContactCommand> for EnsureContactEmailUniqueRule {
    async fn evaluate(
        &self,
        context: &CreateContactCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if self.repo.get_by_email(&context.tenant_id, &context.email).await.is_some() {
            return RuleResult::failed(
                "DUPLICATE_CONTACT_EMAIL",
                format!("Contact with email '{}' already exists in tenant '{}'.", context.email, context.tenant_id),
            );
        }
        RuleResult::success()
    }
}

// ─── Handlers ──────────────────────────────────────────────────────────────

pub struct CreateContactHandler {
    pub repo: InMemoryContactRepository,
}

#[async_trait]
impl RequestHandler<CreateContactCommand> for CreateContactHandler {
    async fn handle(
        &self,
        request: CreateContactCommand,
        _context: &RequestContext,
    ) -> Result<Contact, DispatchError> {
        let contact = Contact {
            id: format!("{:x}", rand_id()),
            tenant_id: request.tenant_id,
            first_name: request.first_name,
            last_name: request.last_name,
            email: request.email,
            company: request.company,
            status: "Lead".into(),
        };
        Ok(self.repo.save(contact).await)
    }
}

pub struct GetContactHandler {
    pub repo: InMemoryContactRepository,
}

#[async_trait]
impl RequestHandler<GetContactQuery> for GetContactHandler {
    async fn handle(
        &self,
        request: GetContactQuery,
        _context: &RequestContext,
    ) -> Result<Option<Contact>, DispatchError> {
        Ok(self.repo.get_by_id(&request.tenant_id, &request.contact_id).await)
    }
}

fn rand_id() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
