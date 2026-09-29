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
pub struct Product {
    pub id: String,
    pub tenant_id: String,
    pub sku: String,
    pub name: String,
    pub price: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct CreateProductCommand {
    pub tenant_id: String,
    pub sku: String,
    pub name: String,
    pub price: f64,
}

impl Request for CreateProductCommand {
    type Response = Product;
}

#[derive(Debug, Clone)]
pub struct GetProductQuery {
    pub tenant_id: String,
    pub product_id: String,
}

impl Request for GetProductQuery {
    type Response = Option<Product>;
}

#[derive(Clone, Default)]
pub struct InMemoryCatalogRepository {
    storage: Arc<RwLock<HashMap<String, Product>>>,
}

impl InMemoryCatalogRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<Product> {
        let store = self.storage.read().await;
        store.get(&format!("{tenant_id}:{id}")).cloned()
    }

    pub async fn get_by_sku(&self, tenant_id: &str, sku: &str) -> Option<Product> {
        let store = self.storage.read().await;
        store
            .values()
            .find(|p| p.tenant_id == tenant_id && p.sku.eq_ignore_ascii_case(sku))
            .cloned()
    }

    pub async fn save(&self, product: Product) -> Product {
        let mut store = self.storage.write().await;
        store.insert(format!("{}:{}", product.tenant_id, product.id), product.clone());
        product
    }
}

// ─── Business Rules ────────────────────────────────────────────────────────

pub struct EnsureProductPricePositiveRule;

#[async_trait]
impl BusinessRule<CreateProductCommand> for EnsureProductPricePositiveRule {
    async fn evaluate(
        &self,
        command: &CreateProductCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if command.price <= 0.0 {
            return RuleResult::failed(
                "INVALID_PRODUCT_PRICE",
                format!("Product price must be positive. Received: {}", command.price),
            );
        }
        RuleResult::success()
    }
}

pub struct EnsureSkuUniqueRule {
    pub repo: InMemoryCatalogRepository,
}

#[async_trait]
impl BusinessRule<CreateProductCommand> for EnsureSkuUniqueRule {
    async fn evaluate(
        &self,
        command: &CreateProductCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if self.repo.get_by_sku(&command.tenant_id, &command.sku).await.is_some() {
            return RuleResult::failed(
                "DUPLICATE_SKU",
                format!("Product with SKU '{}' already exists in tenant '{}'.", command.sku, command.tenant_id),
            );
        }
        RuleResult::success()
    }
}

// ─── Handlers ──────────────────────────────────────────────────────────────

pub struct CreateProductHandler {
    pub repo: InMemoryCatalogRepository,
}

#[async_trait]
impl RequestHandler<CreateProductCommand> for CreateProductHandler {
    async fn handle(
        &self,
        command: CreateProductCommand,
        _context: &RequestContext,
    ) -> Result<Product, DispatchError> {
        let product = Product {
            id: format!("prod_{}", &uuid_short()),
            tenant_id: command.tenant_id,
            sku: command.sku,
            name: command.name,
            price: command.price,
            status: "Active".into(),
        };
        Ok(self.repo.save(product).await)
    }
}

pub struct GetProductHandler {
    pub repo: InMemoryCatalogRepository,
}

#[async_trait]
impl RequestHandler<GetProductQuery> for GetProductHandler {
    async fn handle(
        &self,
        query: GetProductQuery,
        _context: &RequestContext,
    ) -> Result<Option<Product>, DispatchError> {
        Ok(self.repo.get_by_id(&query.tenant_id, &query.product_id).await)
    }
}

fn uuid_short() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:x}", nanos)
}
