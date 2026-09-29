use crate::{ErrorEnvelope, RequestContext, ValidationError};
use async_trait::async_trait;
use std::fmt;

/// Represents the outcome of evaluating a business rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleResult {
    pub is_success: bool,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

impl RuleResult {
    /// Constructs a successful evaluation result.
    pub fn success() -> Self {
        Self {
            is_success: true,
            error_code: None,
            message: None,
        }
    }

    /// Constructs a failed evaluation result with a code and explanation.
    pub fn failed(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            is_success: false,
            error_code: Some(code.into()),
            message: Some(message.into()),
        }
    }
}

/// Error returned when one or more business rules fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessRuleViolation {
    pub error_code: String,
    pub message: String,
}

impl BusinessRuleViolation {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: code.into(),
            message: message.into(),
        }
    }

    /// Converts the violation into a canonical, transport-independent ErrorEnvelope.
    pub fn to_error_envelope(&self, context: &RequestContext) -> Result<ErrorEnvelope, ValidationError> {
        ErrorEnvelope::new(context, &self.error_code, &self.message, false)
    }
}

impl fmt::Display for BusinessRuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "business rule violation [{}]: {}", self.error_code, self.message)
    }
}

impl std::error::Error for BusinessRuleViolation {}

/// Defines an isolated, strongly-typed business rule contract.
/// Rules are deterministic, step-debuggable, and can be evaluated concurrently or in priority sequence.
#[async_trait]
pub trait BusinessRule<TContext>: Send + Sync {
    /// Execution order priority (lower numbers execute first).
    fn priority(&self) -> i32 {
        0
    }

    /// Evaluates the rule against the given context and request metadata.
    async fn evaluate(
        &self,
        context: &TContext,
        request_context: &RequestContext,
    ) -> RuleResult;
}

/// Ordered pipeline of business rules for a specific request or entity context.
#[derive(Default)]
pub struct RulePipeline<TContext> {
    rules: Vec<Box<dyn BusinessRule<TContext>>>,
}

impl<TContext: Send + Sync> RulePipeline<TContext> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a business rule and sorts the pipeline by rule priority.
    pub fn add_rule(mut self, rule: impl BusinessRule<TContext> + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self.rules.sort_by_key(|r| r.priority());
        self
    }

    /// Evaluates all rules sequentially. Halts immediately on the first failure (short-circuit).
    pub async fn evaluate(
        &self,
        context: &TContext,
        request_context: &RequestContext,
    ) -> Result<(), BusinessRuleViolation> {
        for rule in &self.rules {
            let result = rule.evaluate(context, request_context).await;
            if !result.is_success {
                return Err(BusinessRuleViolation {
                    error_code: result.error_code.unwrap_or_else(|| "RULE_VIOLATION".into()),
                    message: result
                        .message
                        .unwrap_or_else(|| "A business rule violation occurred.".into()),
                });
            }
        }
        Ok(())
    }
}
