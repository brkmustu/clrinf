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
pub struct StockItem {
    pub product_id: String,
    pub tenant_id: String,
    pub available_quantity: i32,
    pub reserved_quantity: i32,
}

#[derive(Debug, Clone)]
pub struct SetStockCommand {
    pub tenant_id: String,
    pub product_id: String,
    pub quantity: i32,
}

impl Request for SetStockCommand {
    type Response = StockItem;
}

#[derive(Debug, Clone)]
pub struct ReserveStockCommand {
    pub tenant_id: String,
    pub product_id: String,
    pub quantity: i32,
}

impl Request for ReserveStockCommand {
    type Response = StockItem;
}

#[derive(Debug, Clone)]
pub struct ReleaseStockCommand {
    pub tenant_id: String,
    pub product_id: String,
    pub quantity: i32,
}

impl Request for ReleaseStockCommand {
    type Response = StockItem;
}

#[derive(Debug, Clone)]
pub struct CommitStockCommand {
    pub tenant_id: String,
    pub product_id: String,
    pub quantity: i32,
}

impl Request for CommitStockCommand {
    type Response = StockItem;
}

#[derive(Clone, Default)]
pub struct InMemoryInventoryRepository {
    storage: Arc<RwLock<HashMap<String, StockItem>>>,
}

impl InMemoryInventoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_by_product_id(&self, tenant_id: &str, product_id: &str) -> Option<StockItem> {
        let store = self.storage.read().await;
        store.get(&format!("{tenant_id}:{product_id}")).cloned()
    }

    pub async fn save(&self, item: StockItem) -> StockItem {
        let mut store = self.storage.write().await;
        store.insert(format!("{}:{}", item.tenant_id, item.product_id), item.clone());
        item
    }
}

// ─── Business Rules ────────────────────────────────────────────────────────

pub struct EnsurePositiveQuantityRule;

#[async_trait]
impl BusinessRule<ReserveStockCommand> for EnsurePositiveQuantityRule {
    async fn evaluate(
        &self,
        command: &ReserveStockCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if command.quantity <= 0 {
            return RuleResult::failed(
                "INVALID_QUANTITY",
                format!("Requested quantity must be positive. Received: {}", command.quantity),
            );
        }
        RuleResult::success()
    }
}

pub struct EnsureStockAvailabilityRule {
    pub repo: InMemoryInventoryRepository,
}

#[async_trait]
impl BusinessRule<ReserveStockCommand> for EnsureStockAvailabilityRule {
    async fn evaluate(
        &self,
        command: &ReserveStockCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        let item = self.repo.get_by_product_id(&command.tenant_id, &command.product_id).await;
        match item {
            Some(stock) if stock.available_quantity >= command.quantity => RuleResult::success(),
            Some(stock) => RuleResult::failed(
                "STOCK_INSUFFICIENT",
                format!("Insufficient stock for product '{}'. Available: {}, Requested: {}", command.product_id, stock.available_quantity, command.quantity),
            ),
            None => RuleResult::failed(
                "STOCK_INSUFFICIENT",
                format!("Stock item not found for product '{}'.", command.product_id),
            ),
        }
    }
}

// ─── Handlers ──────────────────────────────────────────────────────────────

pub struct SetStockHandler {
    pub repo: InMemoryInventoryRepository,
}

#[async_trait]
impl RequestHandler<SetStockCommand> for SetStockHandler {
    async fn handle(
        &self,
        command: SetStockCommand,
        _context: &RequestContext,
    ) -> Result<StockItem, DispatchError> {
        let item = StockItem {
            tenant_id: command.tenant_id,
            product_id: command.product_id,
            available_quantity: command.quantity,
            reserved_quantity: 0,
        };
        Ok(self.repo.save(item).await)
    }
}

pub struct ReserveStockHandler {
    pub repo: InMemoryInventoryRepository,
}

#[async_trait]
impl RequestHandler<ReserveStockCommand> for ReserveStockHandler {
    async fn handle(
        &self,
        command: ReserveStockCommand,
        _context: &RequestContext,
    ) -> Result<StockItem, DispatchError> {
        let mut item = self.repo.get_by_product_id(&command.tenant_id, &command.product_id).await
            .ok_or_else(|| DispatchError::NotFound("Stock not found".into()))?;

        item.available_quantity -= command.quantity;
        item.reserved_quantity += command.quantity;
        Ok(self.repo.save(item).await)
    }
}

pub struct ReleaseStockHandler {
    pub repo: InMemoryInventoryRepository,
}

#[async_trait]
impl RequestHandler<ReleaseStockCommand> for ReleaseStockHandler {
    async fn handle(
        &self,
        command: ReleaseStockCommand,
        _context: &RequestContext,
    ) -> Result<StockItem, DispatchError> {
        let mut item = self.repo.get_by_product_id(&command.tenant_id, &command.product_id).await
            .ok_or_else(|| DispatchError::NotFound("Stock not found".into()))?;

        let release_qty = item.reserved_quantity.min(command.quantity);
        item.available_quantity += release_qty;
        item.reserved_quantity -= release_qty;
        Ok(self.repo.save(item).await)
    }
}

pub struct CommitStockHandler {
    pub repo: InMemoryInventoryRepository,
}

#[async_trait]
impl RequestHandler<CommitStockCommand> for CommitStockHandler {
    async fn handle(
        &self,
        command: CommitStockCommand,
        _context: &RequestContext,
    ) -> Result<StockItem, DispatchError> {
        let mut item = self.repo.get_by_product_id(&command.tenant_id, &command.product_id).await
            .ok_or_else(|| DispatchError::NotFound("Stock not found".into()))?;

        let commit_qty = item.reserved_quantity.min(command.quantity);
        item.reserved_quantity -= commit_qty;
        Ok(self.repo.save(item).await)
    }
}
