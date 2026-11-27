use crate::{BusinessRuleViolation, RequestContext, RuleResult};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

static MONOTONIC_ID: AtomicU64 = AtomicU64::new(1);

/// Primary key contract for entities managed by [`CrudStore`].
pub trait CrudId: Clone + Send + Sync + Display + FromStr + Hash + Eq + 'static {
    fn generate_id() -> Self;
}

impl CrudId for String {
    fn generate_id() -> Self {
        uuid::Uuid::new_v4().to_string()
    }
}

impl CrudId for uuid::Uuid {
    fn generate_id() -> Self {
        uuid::Uuid::new_v4()
    }
}

impl CrudId for i32 {
    fn generate_id() -> Self {
        MONOTONIC_ID.fetch_add(1, Ordering::Relaxed) as i32
    }
}

impl CrudId for i64 {
    fn generate_id() -> Self {
        MONOTONIC_ID.fetch_add(1, Ordering::Relaxed) as i64
    }
}

impl CrudId for u64 {
    fn generate_id() -> Self {
        MONOTONIC_ID.fetch_add(1, Ordering::Relaxed)
    }
}

/// Persistence contract for tenant-scoped CRUD entities.
pub trait CrudEntity: Clone + Send + Sync + Serialize + DeserializeOwned + 'static {
    type Id: CrudId;

    fn id(&self) -> Self::Id;
    fn set_id(&mut self, id: Self::Id);
    fn tenant_id(&self) -> &str;
    fn set_tenant_id(&mut self, tenant_id: String);
}

/// Compile-time metadata emitted by `#[derive(ClrinfCrud)]`.
pub trait CrudMetadata: CrudEntity {
    type CreateDto: DeserializeOwned + Send + 'static;
    type UpdateDto: DeserializeOwned + Send + 'static;

    const ENTITY_NAME: &'static str;
    const ROUTE_PREFIX: &'static str;
    const OPERATIONS: &'static [&'static str];
    const SKIPPED_OPERATIONS: &'static [&'static str];

    fn from_create(id: Self::Id, tenant_id: String, dto: Self::CreateDto) -> Self;
    fn apply_update(&mut self, dto: Self::UpdateDto);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrudQuery {
    pub page: usize,
    pub page_size: usize,
    pub search: Option<String>,
}

impl Default for CrudQuery {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            search: None,
        }
    }
}

impl CrudQuery {
    /// Clamps `page >= 1` and `page_size` to `1..=100`.
    pub fn normalized(&self) -> Self {
        Self {
            page: self.page.max(1),
            page_size: self.page_size.clamp(1, 100),
            search: self
                .search
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug)]
pub enum CrudError {
    NotFound(String),
    Conflict(String),
    Validation(String),
    RuleViolation(BusinessRuleViolation),
    Storage(String),
}

impl Display for CrudError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
            Self::RuleViolation(err) => write!(f, "{err}"),
            Self::Storage(msg) => write!(f, "storage error: {msg}"),
        }
    }
}

impl std::error::Error for CrudError {}

impl From<BusinessRuleViolation> for CrudError {
    fn from(value: BusinessRuleViolation) -> Self {
        Self::RuleViolation(value)
    }
}

#[async_trait]
pub trait CrudStore<T: CrudEntity>: Send + Sync + 'static {
    async fn get_by_id(&self, tenant_id: &str, id: &T::Id) -> Result<Option<T>, CrudError>;
    async fn list(&self, tenant_id: &str, query: &CrudQuery) -> Result<PagedResult<T>, CrudError>;
    async fn create(&self, tenant_id: &str, entity: T) -> Result<T, CrudError>;
    async fn update(&self, tenant_id: &str, entity: T) -> Result<T, CrudError>;
    async fn delete(&self, tenant_id: &str, id: &T::Id) -> Result<bool, CrudError>;
}

#[async_trait]
pub trait BeforeCreateRule<T: Send + Sync>: Send + Sync {
    fn priority(&self) -> i32 {
        0
    }
    async fn evaluate(&self, entity: &T, ctx: &RequestContext) -> RuleResult;
}

#[async_trait]
pub trait BeforeUpdateRule<T: Send + Sync>: Send + Sync {
    fn priority(&self) -> i32 {
        0
    }
    async fn evaluate(&self, current: &T, proposed: &T, ctx: &RequestContext) -> RuleResult;
}

#[async_trait]
pub trait BeforeDeleteRule<T: Send + Sync>: Send + Sync {
    fn priority(&self) -> i32 {
        0
    }
    async fn evaluate(&self, entity: &T, ctx: &RequestContext) -> RuleResult;
}

/// Priority-ordered lifecycle rule runner for CRUD operations.
pub struct CrudRuleRunner<T> {
    before_create: Vec<Box<dyn BeforeCreateRule<T>>>,
    before_update: Vec<Box<dyn BeforeUpdateRule<T>>>,
    before_delete: Vec<Box<dyn BeforeDeleteRule<T>>>,
}

impl<T> Default for CrudRuleRunner<T> {
    fn default() -> Self {
        Self {
            before_create: Vec::new(),
            before_update: Vec::new(),
            before_delete: Vec::new(),
        }
    }
}

impl<T: Send + Sync> CrudRuleRunner<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_before_create(mut self, rule: impl BeforeCreateRule<T> + 'static) -> Self {
        self.before_create.push(Box::new(rule));
        self.before_create.sort_by_key(|r| r.priority());
        self
    }

    pub fn with_before_update(mut self, rule: impl BeforeUpdateRule<T> + 'static) -> Self {
        self.before_update.push(Box::new(rule));
        self.before_update.sort_by_key(|r| r.priority());
        self
    }

    pub fn with_before_delete(mut self, rule: impl BeforeDeleteRule<T> + 'static) -> Self {
        self.before_delete.push(Box::new(rule));
        self.before_delete.sort_by_key(|r| r.priority());
        self
    }

    pub async fn run_before_create(
        &self,
        entity: &T,
        ctx: &RequestContext,
    ) -> Result<(), BusinessRuleViolation> {
        for rule in &self.before_create {
            let res = rule.evaluate(entity, ctx).await;
            if !res.is_success {
                return Err(to_violation(res));
            }
        }
        Ok(())
    }

    pub async fn run_before_update(
        &self,
        current: &T,
        proposed: &T,
        ctx: &RequestContext,
    ) -> Result<(), BusinessRuleViolation> {
        for rule in &self.before_update {
            let res = rule.evaluate(current, proposed, ctx).await;
            if !res.is_success {
                return Err(to_violation(res));
            }
        }
        Ok(())
    }

    pub async fn run_before_delete(
        &self,
        entity: &T,
        ctx: &RequestContext,
    ) -> Result<(), BusinessRuleViolation> {
        for rule in &self.before_delete {
            let res = rule.evaluate(entity, ctx).await;
            if !res.is_success {
                return Err(to_violation(res));
            }
        }
        Ok(())
    }
}

fn to_violation(res: RuleResult) -> BusinessRuleViolation {
    BusinessRuleViolation::new(
        res.error_code.unwrap_or_else(|| "RULE_VIOLATION".into()),
        res.message
            .unwrap_or_else(|| "A business rule violation occurred.".into()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Item {
        slug: String,
    }

    struct ProtectedSlugRule {
        prio: i32,
        code: &'static str,
    }

    #[async_trait]
    impl BeforeDeleteRule<Item> for ProtectedSlugRule {
        fn priority(&self) -> i32 {
            self.prio
        }

        async fn evaluate(&self, entity: &Item, _ctx: &RequestContext) -> RuleResult {
            if entity.slug == "protected" {
                RuleResult::failed(self.code, "Protected item cannot be deleted")
            } else {
                RuleResult::success()
            }
        }
    }

    #[test]
    fn query_normalization_and_id_generation() {
        let q = CrudQuery {
            page: 0,
            page_size: 999,
            search: Some("  cats ".into()),
        }
        .normalized();
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 100);
        assert_eq!(q.search.as_deref(), Some("cats"));

        let s1 = String::generate_id();
        let s2 = String::generate_id();
        assert_ne!(s1, s2);
        let n1 = i32::generate_id();
        let n2 = i32::generate_id();
        assert!(n2 > n1);
    }

    #[tokio::test]
    async fn rule_runner_orders_by_priority_and_short_circuits() {
        let runner = CrudRuleRunner::new()
            .with_before_delete(ProtectedSlugRule {
                prio: 20,
                code: "SECOND",
            })
            .with_before_delete(ProtectedSlugRule {
                prio: 5,
                code: "FIRST",
            });
        let ctx = RequestContext::new("tenant-1", "corr-1", "caus-1").unwrap();
        let err = runner
            .run_before_delete(
                &Item {
                    slug: "protected".into(),
                },
                &ctx,
            )
            .await
            .unwrap_err();
        assert_eq!(err.error_code, "FIRST");
        runner
            .run_before_delete(&Item { slug: "ok".into() }, &ctx)
            .await
            .unwrap();
    }
}
