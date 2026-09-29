use clrinf_core::{
    auth::Claims,
    authz::{AuthorizationService, AuthzDecision, OperationClaim},
    dispatcher::{DispatchError, PipelinedHandler, RequestHandler},
    RequestContext,
};
use ecommerce_monolith::{
    catalog::{
        CreateProductCommand, CreateProductHandler, EnsureProductPricePositiveRule,
        EnsureSkuUniqueRule, InMemoryCatalogRepository,
    },
    inventory::{
        EnsurePositiveQuantityRule, EnsureStockAvailabilityRule, InMemoryInventoryRepository,
        ReleaseStockCommand, ReleaseStockHandler, ReserveStockCommand, ReserveStockHandler,
        SetStockCommand, SetStockHandler,
    },
    orders::{
        CancelOrderCommand, CancelOrderHandler, CreateOrderCommand, CreateOrderHandler,
        EnsureOrderItemsNotEmptyRule, InMemoryOrderRepository, MinimumOrderAmountRule, OrderItem,
    },
    security::create_ecommerce_authorizer,
};

#[tokio::test]
async fn test_create_product_success() {
    let repo = InMemoryCatalogRepository::new();
    let handler = PipelinedHandler::new(CreateProductHandler { repo: repo.clone() })
        .with_rule(EnsureProductPricePositiveRule)
        .with_rule(EnsureSkuUniqueRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_1", "cause_1").unwrap();
    let cmd = CreateProductCommand {
        tenant_id: "tenant_acme".into(),
        sku: "SKU-KEYBOARD".into(),
        name: "Mechanical Keyboard".into(),
        price: 120.0,
    };

    let product = handler.execute(cmd, &context).await.unwrap();
    assert_eq!(product.sku, "SKU-KEYBOARD");
    assert_eq!(product.price, 120.0);
    assert_eq!(product.status, "Active");
}

#[tokio::test]
async fn test_create_product_fails_negative_price() {
    let repo = InMemoryCatalogRepository::new();
    let handler = PipelinedHandler::new(CreateProductHandler { repo: repo.clone() })
        .with_rule(EnsureProductPricePositiveRule)
        .with_rule(EnsureSkuUniqueRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_2", "cause_2").unwrap();
    let cmd = CreateProductCommand {
        tenant_id: "tenant_acme".into(),
        sku: "SKU-FREE".into(),
        name: "Zero Price Item".into(),
        price: 0.0,
    };

    let err = handler.execute(cmd, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "INVALID_PRODUCT_PRICE");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_create_product_fails_duplicate_sku() {
    let repo = InMemoryCatalogRepository::new();
    let handler = PipelinedHandler::new(CreateProductHandler { repo: repo.clone() })
        .with_rule(EnsureProductPricePositiveRule)
        .with_rule(EnsureSkuUniqueRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_3", "cause_3").unwrap();
    let cmd1 = CreateProductCommand {
        tenant_id: "tenant_acme".into(),
        sku: "SKU-MOUSE".into(),
        name: "Gaming Mouse".into(),
        price: 60.0,
    };
    handler.execute(cmd1, &context).await.unwrap();

    let cmd2 = CreateProductCommand {
        tenant_id: "tenant_acme".into(),
        sku: "SKU-MOUSE".into(),
        name: "Office Mouse".into(),
        price: 40.0,
    };
    let err = handler.execute(cmd2, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "DUPLICATE_SKU");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_reserve_stock_success() {
    let repo = InMemoryInventoryRepository::new();
    let set_handler = SetStockHandler { repo: repo.clone() };
    let reserve_handler = PipelinedHandler::new(ReserveStockHandler { repo: repo.clone() })
        .with_rule(EnsurePositiveQuantityRule)
        .with_rule(EnsureStockAvailabilityRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_4", "cause_4").unwrap();
    set_handler.handle(SetStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_1".into(),
        quantity: 10,
    }, &context).await.unwrap();

    let updated = reserve_handler.execute(ReserveStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_1".into(),
        quantity: 3,
    }, &context).await.unwrap();

    assert_eq!(updated.available_quantity, 7);
    assert_eq!(updated.reserved_quantity, 3);
}

#[tokio::test]
async fn test_reserve_stock_fails_insufficient() {
    let repo = InMemoryInventoryRepository::new();
    let set_handler = SetStockHandler { repo: repo.clone() };
    let reserve_handler = PipelinedHandler::new(ReserveStockHandler { repo: repo.clone() })
        .with_rule(EnsurePositiveQuantityRule)
        .with_rule(EnsureStockAvailabilityRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_5", "cause_5").unwrap();
    set_handler.handle(SetStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_2".into(),
        quantity: 2,
    }, &context).await.unwrap();

    let err = reserve_handler.execute(ReserveStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_2".into(),
        quantity: 5,
    }, &context).await.unwrap_err();

    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "STOCK_INSUFFICIENT");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_create_order_success() {
    let repo = InMemoryOrderRepository::new();
    let handler = PipelinedHandler::new(CreateOrderHandler { repo: repo.clone() })
        .with_rule(EnsureOrderItemsNotEmptyRule)
        .with_rule(MinimumOrderAmountRule);

    let context = RequestContext::new("tenant_acme", "corr_6", "cause_6").unwrap();
    let cmd = CreateOrderCommand {
        tenant_id: "tenant_acme".into(),
        customer_id: "cust_1".into(),
        items: vec![OrderItem {
            product_id: "prod_kb".into(),
            quantity: 1,
            unit_price: 120.0,
        }],
    };

    let order = handler.execute(cmd, &context).await.unwrap();
    assert_eq!(order.status, "Pending");
    assert_eq!(order.total_amount, 120.0);
}

#[tokio::test]
async fn test_create_order_fails_minimum_amount() {
    let repo = InMemoryOrderRepository::new();
    let handler = PipelinedHandler::new(CreateOrderHandler { repo: repo.clone() })
        .with_rule(EnsureOrderItemsNotEmptyRule)
        .with_rule(MinimumOrderAmountRule);

    let context = RequestContext::new("tenant_acme", "corr_7", "cause_7").unwrap();
    let cmd = CreateOrderCommand {
        tenant_id: "tenant_acme".into(),
        customer_id: "cust_1".into(),
        items: vec![OrderItem {
            product_id: "prod_sticker".into(),
            quantity: 1,
            unit_price: 5.0,
        }],
    };

    let err = handler.execute(cmd, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "MINIMUM_ORDER_AMOUNT_NOT_MET");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_create_order_fails_empty_items() {
    let repo = InMemoryOrderRepository::new();
    let handler = PipelinedHandler::new(CreateOrderHandler { repo: repo.clone() })
        .with_rule(EnsureOrderItemsNotEmptyRule)
        .with_rule(MinimumOrderAmountRule);

    let context = RequestContext::new("tenant_acme", "corr_8", "cause_8").unwrap();
    let cmd = CreateOrderCommand {
        tenant_id: "tenant_acme".into(),
        customer_id: "cust_1".into(),
        items: vec![],
    };

    let err = handler.execute(cmd, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "EMPTY_ORDER_ITEMS");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_order_cancel_and_release_lifecycle() {
    let inv_repo = InMemoryInventoryRepository::new();
    let ord_repo = InMemoryOrderRepository::new();

    let set_handler = SetStockHandler { repo: inv_repo.clone() };
    let reserve_handler = PipelinedHandler::new(ReserveStockHandler { repo: inv_repo.clone() })
        .with_rule(EnsurePositiveQuantityRule)
        .with_rule(EnsureStockAvailabilityRule { repo: inv_repo.clone() });
    let release_handler = ReleaseStockHandler { repo: inv_repo.clone() };

    let order_handler = PipelinedHandler::new(CreateOrderHandler { repo: ord_repo.clone() })
        .with_rule(EnsureOrderItemsNotEmptyRule)
        .with_rule(MinimumOrderAmountRule);
    let cancel_handler = CancelOrderHandler { repo: ord_repo.clone() };

    let context = RequestContext::new("tenant_acme", "corr_9", "cause_9").unwrap();

    // 1. Initial stock: 10
    set_handler.handle(SetStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_headset".into(),
        quantity: 10,
    }, &context).await.unwrap();

    // 2. Reserve 2 items
    let stock_res = reserve_handler.execute(ReserveStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_headset".into(),
        quantity: 2,
    }, &context).await.unwrap();
    assert_eq!(stock_res.available_quantity, 8);
    assert_eq!(stock_res.reserved_quantity, 2);

    // 3. Create order
    let order = order_handler.execute(CreateOrderCommand {
        tenant_id: "tenant_acme".into(),
        customer_id: "cust_1".into(),
        items: vec![OrderItem {
            product_id: "prod_headset".into(),
            quantity: 2,
            unit_price: 50.0,
        }],
    }, &context).await.unwrap();
    assert_eq!(order.status, "Pending");

    // 4. Cancel order
    let cancelled = cancel_handler.handle(CancelOrderCommand {
        tenant_id: "tenant_acme".into(),
        order_id: order.id,
    }, &context).await.unwrap();
    assert_eq!(cancelled.status, "Cancelled");

    // 5. Compensating action: Release stock
    let released = release_handler.handle(ReleaseStockCommand {
        tenant_id: "tenant_acme".into(),
        product_id: "prod_headset".into(),
        quantity: 2,
    }, &context).await.unwrap();
    assert_eq!(released.available_quantity, 10);
    assert_eq!(released.reserved_quantity, 0);
}

#[tokio::test]
async fn test_multi_tenant_authorization() {
    let authorizer = create_ecommerce_authorizer();
    let tenant_beta_claims = Claims {
        sub: "user_beta".into(),
        roles: vec!["Customer".into()],
        tenant_id: "tenant_beta".into(),
        exp: 9999999999,
        iat: 0,
    };

    let target_claim = OperationClaim::with_tenant("Orders", "orders.create", "tenant_acme");

    let decision = authorizer.authorize(&tenant_beta_claims, &[target_claim]).await.unwrap();
    match decision {
        AuthzDecision::Deny { is_tenant_violation, reason } => {
            assert!(is_tenant_violation);
            assert!(reason.contains("Tenant isolation violation"));
        }
        _ => panic!("Expected Deny due to tenant mismatch, got {decision:?}"),
    }
}
