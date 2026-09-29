use async_trait::async_trait;
use clrinf_core::{
    BusinessRule, DispatchError, PipelinedHandler, Request, RequestHandler, RequestContext,
    RuleResult,
};

#[derive(Clone, Debug)]
struct CreateProduct {
    name: String,
    price: f64,
}

impl Request for CreateProduct {
    type Response = String;
}

struct CreateProductHandler;

#[async_trait]
impl RequestHandler<CreateProduct> for CreateProductHandler {
    async fn handle(&self, request: CreateProduct, _context: &RequestContext) -> Result<String, DispatchError> {
        Ok(format!("product_created:{}", request.name))
    }
}

struct NameMustNotBeEmptyRule;

#[async_trait]
impl BusinessRule<CreateProduct> for NameMustNotBeEmptyRule {
    fn priority(&self) -> i32 {
        1
    }

    async fn evaluate(&self, context: &CreateProduct, _request_context: &RequestContext) -> RuleResult {
        if context.name.trim().is_empty() {
            RuleResult::failed("EMPTY_NAME", "Product name cannot be empty")
        } else {
            RuleResult::success()
        }
    }
}

struct PriceMustBePositiveRule;

#[async_trait]
impl BusinessRule<CreateProduct> for PriceMustBePositiveRule {
    fn priority(&self) -> i32 {
        2
    }

    async fn evaluate(&self, context: &CreateProduct, _request_context: &RequestContext) -> RuleResult {
        if context.price <= 0.0 {
            RuleResult::failed("INVALID_PRICE", "Product price must be positive")
        } else {
            RuleResult::success()
        }
    }
}

struct EnterpriseTenantLimitRule;

#[async_trait]
impl BusinessRule<CreateProduct> for EnterpriseTenantLimitRule {
    fn priority(&self) -> i32 {
        3
    }

    async fn evaluate(&self, context: &CreateProduct, request_context: &RequestContext) -> RuleResult {
        if request_context.tenant_id == "tenant-enterprise" && context.price > 10_000.0 {
            RuleResult::failed("PRICE_EXCEEDED", "Enterprise price limit is 10,000")
        } else {
            RuleResult::success()
        }
    }
}

#[tokio::test]
async fn test_when_all_rules_pass_handler_returns_success() {
    let context = RequestContext::new("tenant-standard", "corr-1", "cause-1").unwrap();
    let handler = PipelinedHandler::new(CreateProductHandler)
        .with_rule(NameMustNotBeEmptyRule)
        .with_rule(PriceMustBePositiveRule)
        .with_rule(EnterpriseTenantLimitRule);

    let request = CreateProduct {
        name: "RustBook".into(),
        price: 49.99,
    };

    let result = handler.execute(request, &context).await;
    assert_eq!(result, Ok("product_created:RustBook".into()));
}

#[tokio::test]
async fn test_when_rule_fails_execution_halts_with_error_code() {
    let context = RequestContext::new("tenant-standard", "corr-1", "cause-1").unwrap();
    let handler = PipelinedHandler::new(CreateProductHandler)
        .with_rule(NameMustNotBeEmptyRule)
        .with_rule(PriceMustBePositiveRule);

    let invalid_name = CreateProduct {
        name: "".into(),
        price: 49.99,
    };

    let err = handler.execute(invalid_name, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "EMPTY_NAME");
            assert_eq!(v.message, "Product name cannot be empty");

            let envelope = v.to_error_envelope(&context).unwrap();
            assert_eq!(envelope.error_code, "EMPTY_NAME");
            assert_eq!(envelope.tenant_id, "tenant-standard");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_multi_tenant_rule_enforcement() {
    let enterprise_context = RequestContext::new("tenant-enterprise", "corr-1", "cause-1").unwrap();
    let standard_context = RequestContext::new("tenant-standard", "corr-2", "cause-2").unwrap();

    let handler = PipelinedHandler::new(CreateProductHandler)
        .with_rule(EnterpriseTenantLimitRule);

    let expensive_product = CreateProduct {
        name: "Mainframe".into(),
        price: 50_000.0,
    };

    // Fails for enterprise tenant
    let err = handler.execute(expensive_product.clone(), &enterprise_context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => assert_eq!(v.error_code, "PRICE_EXCEEDED"),
        _ => panic!("Expected RuleViolation"),
    }

    // Passes for standard tenant
    let res = handler.execute(expensive_product, &standard_context).await;
    assert!(res.is_ok());
}
