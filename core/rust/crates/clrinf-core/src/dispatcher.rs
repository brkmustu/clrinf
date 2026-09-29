use crate::{
    rules::{BusinessRule, BusinessRuleViolation, RulePipeline},
    ErrorEnvelope, RequestContext, ValidationError,
};
use async_trait::async_trait;
use std::fmt;

/// Defines a CQRS Request message returning a strongly-typed response.
pub trait Request: Send + Sync + 'static {
    type Response: Send + Sync + 'static;
}

/// Dispatches requests to registered handlers through business rule validation pipelines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    RuleViolation(BusinessRuleViolation),
    Validation(ValidationError),
    Execution(String),
    NotFound(String),
    Unauthorized(String),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuleViolation(v) => write!(f, "{v}"),
            Self::Validation(v) => write!(f, "validation error: {v}"),
            Self::Execution(e) => write!(f, "execution error: {e}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Unauthorized(msg) => write!(f, "unauthorized: {msg}"),
        }
    }
}

impl std::error::Error for DispatchError {}

impl From<BusinessRuleViolation> for DispatchError {
    fn from(v: BusinessRuleViolation) -> Self {
        Self::RuleViolation(v)
    }
}

impl From<ValidationError> for DispatchError {
    fn from(v: ValidationError) -> Self {
        Self::Validation(v)
    }
}

impl DispatchError {
    pub fn to_error_envelope(&self, context: &RequestContext) -> Result<ErrorEnvelope, ValidationError> {
        match self {
            Self::RuleViolation(v) => v.to_error_envelope(context),
            Self::Validation(v) => ErrorEnvelope::new(context, "VALIDATION_FAILED", &v.0, false),
            Self::Execution(msg) => ErrorEnvelope::new(context, "INTERNAL_ERROR", msg, false),
            Self::NotFound(msg) => ErrorEnvelope::new(context, "NOT_FOUND", msg, false),
            Self::Unauthorized(msg) => ErrorEnvelope::new(context, "UNAUTHORIZED", msg, false),
        }
    }
}

/// Handles a specific Request type.
#[async_trait]
pub trait RequestHandler<R: Request>: Send + Sync + 'static {
    async fn handle(&self, request: R, context: &RequestContext) -> Result<R::Response, DispatchError>;
}

/// Wraps a RequestHandler with an isolated business rule evaluation pipeline.
/// Executes all rules in priority order before invoking the inner handler.
pub struct PipelinedHandler<R: Request, H: RequestHandler<R>> {
    handler: H,
    rules: RulePipeline<R>,
}

impl<R: Request, H: RequestHandler<R>> PipelinedHandler<R, H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            rules: RulePipeline::new(),
        }
    }

    pub fn with_rule(mut self, rule: impl BusinessRule<R> + 'static) -> Self {
        self.rules = self.rules.add_rule(rule);
        self
    }

    pub async fn execute(&self, request: R, context: &RequestContext) -> Result<R::Response, DispatchError> {
        context.validate()?;
        self.rules.evaluate(&request, context).await?;
        self.handler.handle(request, context).await
    }
}
