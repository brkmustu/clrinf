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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Deal {
    pub id: String,
    pub tenant_id: String,
    pub title: String,
    pub contact_id: String,
    pub amount: f64,
    pub stage: String,
    pub probability: u32,
}

#[derive(Debug, Clone)]
pub struct CreateDealCommand {
    pub tenant_id: String,
    pub title: String,
    pub contact_id: String,
    pub amount: f64,
}

impl Request for CreateDealCommand {
    type Response = Deal;
}

#[derive(Debug, Clone)]
pub struct ChangeDealStageCommand {
    pub tenant_id: String,
    pub deal_id: String,
    pub new_stage: String,
}

impl Request for ChangeDealStageCommand {
    type Response = Deal;
}

#[derive(Debug, Clone)]
pub struct GetDealQuery {
    pub tenant_id: String,
    pub deal_id: String,
}

impl Request for GetDealQuery {
    type Response = Option<Deal>;
}

#[derive(Clone, Default)]
pub struct InMemoryDealRepository {
    storage: Arc<RwLock<HashMap<String, Deal>>>,
}

impl InMemoryDealRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<Deal> {
        let store = self.storage.read().await;
        store.get(&format!("{tenant_id}:{id}")).cloned()
    }

    pub async fn save(&self, deal: Deal) -> Deal {
        let mut store = self.storage.write().await;
        store.insert(format!("{}:{}", deal.tenant_id, deal.id), deal.clone());
        deal
    }
}

// ─── Business Rules ────────────────────────────────────────────────────────

pub struct EnsureDealAmountPositiveRule;

#[async_trait]
impl BusinessRule<CreateDealCommand> for EnsureDealAmountPositiveRule {
    async fn evaluate(
        &self,
        context: &CreateDealCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if context.amount <= 0.0 {
            return RuleResult::failed(
                "INVALID_DEAL_AMOUNT",
                "Deal amount must be greater than zero.",
            );
        }
        RuleResult::success()
    }
}

pub struct StageProgressionRule {
    pub repo: InMemoryDealRepository,
}

#[async_trait]
impl BusinessRule<ChangeDealStageCommand> for StageProgressionRule {
    async fn evaluate(
        &self,
        context: &ChangeDealStageCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        let valid_stages = ["Prospect", "Qualified", "Proposal", "Negotiation", "ClosedWon", "ClosedLost"];
        if !valid_stages.contains(&context.new_stage.as_str()) {
            return RuleResult::failed(
                "UNKNOWN_DEAL_STAGE",
                format!("Stage '{}' is not a recognized CRM deal stage.", context.new_stage),
            );
        }

        let deal = match self.repo.get_by_id(&context.tenant_id, &context.deal_id).await {
            Some(d) => d,
            None => {
                return RuleResult::failed(
                    "DEAL_NOT_FOUND",
                    format!("Deal '{}' not found in tenant '{}'.", context.deal_id, context.tenant_id),
                );
            }
        };

        if deal.stage == "ClosedWon" || deal.stage == "ClosedLost" {
            return RuleResult::failed(
                "DEAL_ALREADY_CLOSED",
                format!("Deal is already finalized as '{}' and cannot transition directly.", deal.stage),
            );
        }

        if deal.stage == "Prospect" && context.new_stage == "ClosedWon" {
            return RuleResult::failed(
                "STAGE_PROGRESSION_VIOLATION",
                "Cannot transition deal directly from 'Prospect' to 'ClosedWon' without a Proposal stage.",
            );
        }

        RuleResult::success()
    }
}

// ─── Handlers ──────────────────────────────────────────────────────────────

pub struct CreateDealHandler {
    pub repo: InMemoryDealRepository,
}

#[async_trait]
impl RequestHandler<CreateDealCommand> for CreateDealHandler {
    async fn handle(
        &self,
        request: CreateDealCommand,
        _context: &RequestContext,
    ) -> Result<Deal, DispatchError> {
        let deal = Deal {
            id: format!("{:x}", rand_id()),
            tenant_id: request.tenant_id,
            title: request.title,
            contact_id: request.contact_id,
            amount: request.amount,
            stage: "Prospect".into(),
            probability: 10,
        };
        Ok(self.repo.save(deal).await)
    }
}

pub struct ChangeDealStageHandler {
    pub repo: InMemoryDealRepository,
}

#[async_trait]
impl RequestHandler<ChangeDealStageCommand> for ChangeDealStageHandler {
    async fn handle(
        &self,
        request: ChangeDealStageCommand,
        _context: &RequestContext,
    ) -> Result<Deal, DispatchError> {
        let mut deal = self
            .repo
            .get_by_id(&request.tenant_id, &request.deal_id)
            .await
            .ok_or_else(|| DispatchError::NotFound(format!("Deal {}", request.deal_id)))?;

        deal.stage = request.new_stage.clone();
        deal.probability = match request.new_stage.as_str() {
            "Prospect" => 10,
            "Qualified" => 30,
            "Proposal" => 60,
            "Negotiation" => 80,
            "ClosedWon" => 100,
            "ClosedLost" => 0,
            _ => deal.probability,
        };

        Ok(self.repo.save(deal).await)
    }
}

pub struct GetDealHandler {
    pub repo: InMemoryDealRepository,
}

#[async_trait]
impl RequestHandler<GetDealQuery> for GetDealHandler {
    async fn handle(
        &self,
        request: GetDealQuery,
        _context: &RequestContext,
    ) -> Result<Option<Deal>, DispatchError> {
        Ok(self.repo.get_by_id(&request.tenant_id, &request.deal_id).await)
    }
}

fn rand_id() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
