use crate::domain::Category;
use async_trait::async_trait;
use clrinf_core::{crud::BeforeDeleteRule, RequestContext, RuleResult};

pub struct ActiveProductsCheck;

#[async_trait]
impl BeforeDeleteRule<Category> for ActiveProductsCheck {
    async fn evaluate(&self, entity: &Category, _ctx: &RequestContext) -> RuleResult {
        if entity.slug.eq_ignore_ascii_case("protected") {
            RuleResult::failed(
                "CATEGORY_PROTECTED",
                "Cannot delete protected category with active products.",
            )
        } else {
            RuleResult::success()
        }
    }
}
