pub mod auth;
pub mod authz;
pub mod cache;
pub mod context;
pub mod dispatcher;
pub mod envelope;
pub mod error;
pub mod idempotency;
pub mod logging;
pub mod outbox;
pub mod persistence;
pub mod rules;
pub mod transaction;

pub use context::{RequestContext, ValidationError};
pub use dispatcher::{DispatchError, PipelinedHandler, Request, RequestHandler};
pub use envelope::EventEnvelope;
pub use error::ErrorEnvelope;
pub use rules::{BusinessRule, BusinessRuleViolation, RulePipeline, RuleResult};
