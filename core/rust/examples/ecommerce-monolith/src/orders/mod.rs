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
pub struct OrderItem {
    pub product_id: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub id: String,
    pub tenant_id: String,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct CreateOrderCommand {
    pub tenant_id: String,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
}

impl Request for CreateOrderCommand {
    type Response = Order;
}

#[derive(Debug, Clone)]
pub struct CancelOrderCommand {
    pub tenant_id: String,
    pub order_id: String,
}

impl Request for CancelOrderCommand {
    type Response = Order;
}

#[derive(Debug, Clone)]
pub struct CompleteOrderCommand {
    pub tenant_id: String,
    pub order_id: String,
}

impl Request for CompleteOrderCommand {
    type Response = Order;
}

#[derive(Clone, Default)]
pub struct InMemoryOrderRepository {
    storage: Arc<RwLock<HashMap<String, Order>>>,
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<Order> {
        let store = self.storage.read().await;
        store.get(&format!("{tenant_id}:{id}")).cloned()
    }

    pub async fn save(&self, order: Order) -> Order {
        let mut store = self.storage.write().await;
        store.insert(format!("{}:{}", order.tenant_id, order.id), order.clone());
        order
    }
}

// ─── Business Rules ────────────────────────────────────────────────────────

pub struct EnsureOrderItemsNotEmptyRule;

#[async_trait]
impl BusinessRule<CreateOrderCommand> for EnsureOrderItemsNotEmptyRule {
    async fn evaluate(
        &self,
        command: &CreateOrderCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        if command.items.is_empty() {
            return RuleResult::failed("EMPTY_ORDER_ITEMS", "Order must contain at least one item.");
        }
        RuleResult::success()
    }
}

pub struct MinimumOrderAmountRule;

#[async_trait]
impl BusinessRule<CreateOrderCommand> for MinimumOrderAmountRule {
    async fn evaluate(
        &self,
        command: &CreateOrderCommand,
        _request_context: &RequestContext,
    ) -> RuleResult {
        let total: f64 = command.items.iter().map(|i| i.quantity as f64 * i.unit_price).sum();
        if total < 25.0 {
            return RuleResult::failed(
                "MINIMUM_ORDER_AMOUNT_NOT_MET",
                format!("Order total ${:.2} is below minimum required threshold $25.00.", total),
            );
        }
        RuleResult::success()
    }
}

// ─── Handlers ──────────────────────────────────────────────────────────────

pub struct CreateOrderHandler {
    pub repo: InMemoryOrderRepository,
}

#[async_trait]
impl RequestHandler<CreateOrderCommand> for CreateOrderHandler {
    async fn handle(
        &self,
        command: CreateOrderCommand,
        _context: &RequestContext,
    ) -> Result<Order, DispatchError> {
        let total: f64 = command.items.iter().map(|i| i.quantity as f64 * i.unit_price).sum();
        let order = Order {
            id: format!("ord_{}", &uuid_short()),
            tenant_id: command.tenant_id,
            customer_id: command.customer_id,
            items: command.items,
            total_amount: total,
            status: "Pending".into(),
        };
        Ok(self.repo.save(order).await)
    }
}

pub struct CancelOrderHandler {
    pub repo: InMemoryOrderRepository,
}

#[async_trait]
impl RequestHandler<CancelOrderCommand> for CancelOrderHandler {
    async fn handle(
        &self,
        command: CancelOrderCommand,
        _context: &RequestContext,
    ) -> Result<Order, DispatchError> {
        let mut order = self.repo.get_by_id(&command.tenant_id, &command.order_id).await
            .ok_or_else(|| DispatchError::NotFound("Order not found".into()))?;

        order.status = "Cancelled".into();
        Ok(self.repo.save(order).await)
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
