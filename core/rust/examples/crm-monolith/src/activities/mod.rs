use crate::deals::InMemoryDealRepository;
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
pub struct Activity {
    pub id: String,
    pub tenant_id: String,
    pub deal_id: Option<String>,
    pub contact_id: Option<String>,
    pub activity_type: String,
    pub subject: String,
    pub notes: String,
    pub is_completed: bool,
}

#[derive(Debug, Clone)]
pub struct LogActivityCommand {
    pub tenant_id: String,
    pub deal_id: Option<String>,
    pub contact_id: Option<String>,
    pub activity_type: String,
    pub subject: String,
    pub notes: String,
}

impl Request for LogActivityCommand {
    type Response = Activity;
}

#[derive(Debug, Clone)]
pub struct CompleteActivityCommand {
    pub tenant_id: String,
    pub activity_id: String,
}

impl Request for CompleteActivityCommand {
    type Response = Activity;
}

#[derive(Clone, Default)]
pub struct InMemoryActivityRepository {
    storage: Arc<RwLock<HashMap<String, Activity>>>,
}

impl InMemoryActivityRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<Activity> {
        let store = self.storage.read().await;
        store.get(&format!("{tenant_id}:{id}")).cloned()
    }

    pub async fn save(&self, activity: Activity) -> Activity {
        let mut store = self.storage.write().await;
        store.insert(format!("{}:{}", activity.tenant_id, activity.id), activity.clone());
        activity
    }
}

// ─── Business Rules ────────────────────────────────────────────────────────

pub struct CannotLogActivityOnLostDealRule {
    pub deal_repo: InMemoryDealRepository,
}

#[async_trait]
impl BusinessRule<LogActivityCommand> for CannotLogActivityOnLostDealRule {
    async fn evaluate(
        &self,
        context: &LogActivityCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if let Some(ref deal_id) = context.deal_id {
            if let Some(deal) = self.deal_repo.get_by_id(&context.tenant_id, deal_id).await {
                if deal.stage == "ClosedLost" {
                    return RuleResult::failed(
                        "CANNOT_LOG_ON_LOST_DEAL",
                        format!("Cannot log new activity on deal '{deal_id}' because it is marked as 'ClosedLost'."),
                    );
                }
            }
        }
        RuleResult::success()
    }
}

// ─── Handlers ──────────────────────────────────────────────────────────────

pub struct LogActivityHandler {
    pub repo: InMemoryActivityRepository,
}

#[async_trait]
impl RequestHandler<LogActivityCommand> for LogActivityHandler {
    async fn handle(
        &self,
        request: LogActivityCommand,
        _context: &RequestContext,
    ) -> Result<Activity, DispatchError> {
        let activity = Activity {
            id: format!("{:x}", rand_id()),
            tenant_id: request.tenant_id,
            deal_id: request.deal_id,
            contact_id: request.contact_id,
            activity_type: request.activity_type,
            subject: request.subject,
            notes: request.notes,
            is_completed: false,
        };
        Ok(self.repo.save(activity).await)
    }
}

pub struct CompleteActivityHandler {
    pub repo: InMemoryActivityRepository,
}

#[async_trait]
impl RequestHandler<CompleteActivityCommand> for CompleteActivityHandler {
    async fn handle(
        &self,
        request: CompleteActivityCommand,
        _context: &RequestContext,
    ) -> Result<Activity, DispatchError> {
        let mut activity = self
            .repo
            .get_by_id(&request.tenant_id, &request.activity_id)
            .await
            .ok_or_else(|| DispatchError::NotFound(format!("Activity {}", request.activity_id)))?;

        activity.is_completed = true;
        Ok(self.repo.save(activity).await)
    }
}

fn rand_id() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
