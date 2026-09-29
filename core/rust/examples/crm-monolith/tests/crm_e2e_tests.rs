use clrinf_core::{
    auth::Claims,
    authz::{AuthorizationService, AuthzDecision, OperationClaim},
    dispatcher::{DispatchError, PipelinedHandler, RequestHandler},
    RequestContext,
};
use crm_monolith::{
    activities::{
        CannotLogActivityOnLostDealRule, InMemoryActivityRepository, LogActivityCommand,
        LogActivityHandler,
    },
    contacts::{
        Contact, CreateContactCommand, CreateContactHandler, EnsureContactEmailUniqueRule,
        GetContactHandler, GetContactQuery, InMemoryContactRepository,
        ValidateContactEmailFormatRule,
    },
    deals::{
        ChangeDealStageCommand, ChangeDealStageHandler, CreateDealCommand, CreateDealHandler,
        EnsureDealAmountPositiveRule, InMemoryDealRepository, StageProgressionRule,
    },
    security::create_crm_authorizer,
};
use std::time::Instant;

#[tokio::test]
async fn test_create_contact_with_pipelined_handler_success() {
    let repo = InMemoryContactRepository::new();
    let handler = PipelinedHandler::new(CreateContactHandler { repo: repo.clone() })
        .with_rule(ValidateContactEmailFormatRule)
        .with_rule(EnsureContactEmailUniqueRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_1", "cause_1").unwrap();
    let cmd = CreateContactCommand {
        tenant_id: "tenant_acme".into(),
        first_name: "Clark".into(),
        last_name: "Kent".into(),
        email: "clark@dailyplanet.com".into(),
        company: "Daily Planet".into(),
    };

    let contact = handler.execute(cmd, &context).await.unwrap();
    assert_eq!(contact.first_name, "Clark");
    assert_eq!(contact.email, "clark@dailyplanet.com");
    assert_eq!(contact.tenant_id, "tenant_acme");
    assert_eq!(contact.status, "Lead");
}

#[tokio::test]
async fn test_create_contact_fails_when_email_format_invalid() {
    let repo = InMemoryContactRepository::new();
    let handler = PipelinedHandler::new(CreateContactHandler { repo: repo.clone() })
        .with_rule(ValidateContactEmailFormatRule)
        .with_rule(EnsureContactEmailUniqueRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_2", "cause_2").unwrap();
    let cmd = CreateContactCommand {
        tenant_id: "tenant_acme".into(),
        first_name: "Bad".into(),
        last_name: "Email".into(),
        email: "not-an-email-address".into(),
        company: "Daily Planet".into(),
    };

    let err = handler.execute(cmd, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "INVALID_EMAIL_FORMAT");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_create_contact_fails_when_email_duplicate() {
    let repo = InMemoryContactRepository::new();
    let handler = PipelinedHandler::new(CreateContactHandler { repo: repo.clone() })
        .with_rule(ValidateContactEmailFormatRule)
        .with_rule(EnsureContactEmailUniqueRule { repo: repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_3", "cause_3").unwrap();
    let cmd1 = CreateContactCommand {
        tenant_id: "tenant_acme".into(),
        first_name: "Lois".into(),
        last_name: "Lane".into(),
        email: "lois@dailyplanet.com".into(),
        company: "Daily Planet".into(),
    };
    handler.execute(cmd1, &context).await.unwrap();

    let cmd2 = CreateContactCommand {
        tenant_id: "tenant_acme".into(),
        first_name: "Lois2".into(),
        last_name: "Lane".into(),
        email: "lois@dailyplanet.com".into(),
        company: "Daily Planet".into(),
    };

    let err = handler.execute(cmd2, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "DUPLICATE_CONTACT_EMAIL");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_deal_stage_progression_rule_prevents_direct_jump() {
    let deal_repo = InMemoryDealRepository::new();
    let create_handler = PipelinedHandler::new(CreateDealHandler { repo: deal_repo.clone() })
        .with_rule(EnsureDealAmountPositiveRule);

    let stage_handler = PipelinedHandler::new(ChangeDealStageHandler { repo: deal_repo.clone() })
        .with_rule(StageProgressionRule { repo: deal_repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_4", "cause_4").unwrap();

    // 1. Create deal in Prospect stage
    let create_cmd = CreateDealCommand {
        tenant_id: "tenant_acme".into(),
        title: "Metropolis Tower Deal".into(),
        contact_id: "c_1".into(),
        amount: 150000.0,
    };
    let deal = create_handler.execute(create_cmd, &context).await.unwrap();
    assert_eq!(deal.stage, "Prospect");

    // 2. Direct jump from Prospect to ClosedWon must fail
    let jump_cmd = ChangeDealStageCommand {
        tenant_id: "tenant_acme".into(),
        deal_id: deal.id.clone(),
        new_stage: "ClosedWon".into(),
    };
    let err = stage_handler.execute(jump_cmd, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "STAGE_PROGRESSION_VIOLATION");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }

    // 3. Normal progression: Prospect -> Proposal -> ClosedWon
    let proposal_cmd = ChangeDealStageCommand {
        tenant_id: "tenant_acme".into(),
        deal_id: deal.id.clone(),
        new_stage: "Proposal".into(),
    };
    let proposal_deal = stage_handler.execute(proposal_cmd, &context).await.unwrap();
    assert_eq!(proposal_deal.stage, "Proposal");

    let win_cmd = ChangeDealStageCommand {
        tenant_id: "tenant_acme".into(),
        deal_id: deal.id.clone(),
        new_stage: "ClosedWon".into(),
    };
    let won_deal = stage_handler.execute(win_cmd, &context).await.unwrap();
    assert_eq!(won_deal.stage, "ClosedWon");
    assert_eq!(won_deal.probability, 100);
}

#[tokio::test]
async fn test_cannot_log_activity_on_lost_deal() {
    let deal_repo = InMemoryDealRepository::new();
    let activity_repo = InMemoryActivityRepository::new();

    let create_deal_handler = PipelinedHandler::new(CreateDealHandler { repo: deal_repo.clone() })
        .with_rule(EnsureDealAmountPositiveRule);

    let stage_handler = PipelinedHandler::new(ChangeDealStageHandler { repo: deal_repo.clone() })
        .with_rule(StageProgressionRule { repo: deal_repo.clone() });

    let activity_handler = PipelinedHandler::new(LogActivityHandler { repo: activity_repo.clone() })
        .with_rule(CannotLogActivityOnLostDealRule { deal_repo: deal_repo.clone() });

    let context = RequestContext::new("tenant_acme", "corr_5", "cause_5").unwrap();

    // 1. Create deal and mark ClosedLost
    let deal = create_deal_handler
        .execute(
            CreateDealCommand {
                tenant_id: "tenant_acme".into(),
                title: "Failed Pitch".into(),
                contact_id: "c_2".into(),
                amount: 30000.0,
            },
            &context,
        )
        .await
        .unwrap();

    stage_handler
        .execute(
            ChangeDealStageCommand {
                tenant_id: "tenant_acme".into(),
                deal_id: deal.id.clone(),
                new_stage: "ClosedLost".into(),
            },
            &context,
        )
        .await
        .unwrap();

    // 2. Attempt to log activity on lost deal
    let act_cmd = LogActivityCommand {
        tenant_id: "tenant_acme".into(),
        deal_id: Some(deal.id.clone()),
        contact_id: Some("c_2".into()),
        activity_type: "Meeting".into(),
        subject: "Review".into(),
        notes: "Why did we lose?".into(),
    };

    let err = activity_handler.execute(act_cmd, &context).await.unwrap_err();
    match err {
        DispatchError::RuleViolation(v) => {
            assert_eq!(v.error_code, "CANNOT_LOG_ON_LOST_DEAL");
        }
        _ => panic!("Expected RuleViolation, got {err:?}"),
    }
}

#[tokio::test]
async fn test_cedar_forbid_guard_denies_cross_tenant_access() {
    let authorizer = create_crm_authorizer();

    // User is in tenant_acme, role SalesManager
    let claims = Claims {
        sub: "user_sales".into(),
        tenant_id: "tenant_acme".into(),
        roles: vec!["SalesManager".into()],
        exp: 9999999999,
        iat: 0,
    };

    // Operation targets resource in tenant_luthorcorp
    let op = OperationClaim::with_tenant("Contacts", "contacts.create", "tenant_luthorcorp");

    let decision = authorizer.authorize(&claims, &[op]).await.unwrap();
    match decision {
        AuthzDecision::Deny { is_tenant_violation, reason } => {
            assert!(is_tenant_violation);
            assert!(reason.contains("Tenant isolation violation"));
        }
        _ => panic!("Expected Deny with tenant violation!"),
    }
}

#[tokio::test]
async fn test_cedar_platform_admin_bypasses_tenant_isolation() {
    let authorizer = create_crm_authorizer();

    let claims = Claims {
        sub: "super_admin".into(),
        tenant_id: "system".into(),
        roles: vec!["PlatformAdmin".into()],
        exp: 9999999999,
        iat: 0,
    };

    let op = OperationClaim::with_tenant("Contacts", "contacts.create", "tenant_luthorcorp");

    let decision = authorizer.authorize(&claims, &[op]).await.unwrap();
    match decision {
        AuthzDecision::Allow { .. } => {}
        _ => panic!("Expected Allow for PlatformAdmin!"),
    }
}

#[tokio::test]
async fn test_performance_benchmark_10000_operations() {
    let repo = InMemoryContactRepository::new();
    let handler = GetContactHandler { repo: repo.clone() };

    let contact = Contact {
        id: "c_fast".into(),
        tenant_id: "tenant_acme".into(),
        first_name: "Barry".into(),
        last_name: "Allen".into(),
        email: "barry@starlabs.com".into(),
        company: "STAR Labs".into(),
        status: "Customer".into(),
    };
    repo.save(contact).await;

    let context = RequestContext::new("tenant_acme", "corr_bench", "cause_bench").unwrap();
    let query = GetContactQuery {
        tenant_id: "tenant_acme".into(),
        contact_id: "c_fast".into(),
    };

    // Warm up
    for _ in 0..100 {
        let _ = handler.handle(query.clone(), &context).await.unwrap();
    }

    let start = Instant::now();
    for _ in 0..10_000 {
        let res = handler.handle(query.clone(), &context).await.unwrap();
        assert!(res.is_some());
    }
    let elapsed = start.elapsed();

    // In Rust, 10,000 in-memory async dispatches take only ~3-10 milliseconds!
    assert!(
        elapsed.as_millis() < 500,
        "Expected <500ms, actual: {:?}",
        elapsed
    );
}
