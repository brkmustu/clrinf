use clrinf_adapters::MemoryWorkflowStore;
use clrinf_core::{
    persistence::{Claim, IdempotencyStore, Lease, OutboxStore, StoreError, WorkflowStore},
    EventEnvelope, RequestContext,
};
use serde_json::json;
use std::sync::{Arc, Barrier};

fn claim(store: &MemoryWorkflowStore, tenant: &str, key: &str, now: u64) -> Lease {
    match store.claim(tenant, key, "payload", now, 10).unwrap() {
        Claim::Acquired(lease) => lease,
        other => panic!("expected lease, got {other:?}"),
    }
}

fn event(tenant: &str) -> EventEnvelope {
    EventEnvelope::new(
        &RequestContext::root(tenant).unwrap(),
        "test",
        "Changed",
        Default::default(),
    )
    .unwrap()
}

#[test]
fn replay_conflict_and_tenant_isolation() {
    let store = MemoryWorkflowStore::default();
    let first = claim(&store, "one", "key", 100);
    assert_eq!(
        store.claim("one", "key", "payload", 101, 10).unwrap(),
        Claim::InProgress
    );
    assert_eq!(
        store.claim("one", "key", "different", 101, 10).unwrap(),
        Claim::Conflict
    );
    let second = claim(&store, "two", "key", 100);
    let first_event = event("one");
    store
        .commit(&first, 101, json!({"ok": true}), vec![first_event.clone()])
        .unwrap();
    store
        .commit(&second, 101, json!(null), vec![event("two")])
        .unwrap();
    assert_eq!(
        store.claim("one", "key", "payload", 999, 10).unwrap(),
        Claim::Replay(json!({"ok": true}))
    );
    assert_eq!(
        store.claim("two", "key", "payload", 999, 10).unwrap(),
        Claim::Replay(json!(null))
    );
    assert_eq!(store.pending("one", 1).unwrap(), vec![first_event.clone()]);
    assert_eq!(
        store.mark_published("two", &first_event.id),
        Err(StoreError::NotFound)
    );
    store.mark_published("one", &first_event.id).unwrap();
    store.mark_published("one", &first_event.id).unwrap();
    assert!(store.pending("one", 100).unwrap().is_empty());
    assert_eq!(store.pending("two", 100).unwrap().len(), 1);
}

#[test]
fn expiry_fences_old_workers_and_abandon_allows_retry() {
    let store = MemoryWorkflowStore::default();
    let expired = claim(&store, "one", "key", 100);
    assert_eq!(
        store.commit(&expired, 110, json!(true), vec![]),
        Err(StoreError::LeaseLost)
    );
    let current = claim(&store, "one", "key", 110);
    assert_ne!(expired.token, current.token);
    assert_eq!(
        store.commit(&expired, 111, json!(true), vec![]),
        Err(StoreError::LeaseLost)
    );
    assert_eq!(store.abandon(&expired), Err(StoreError::LeaseLost));
    store.abandon(&current).unwrap();
    let retry = claim(&store, "one", "key", 111);
    store.commit(&retry, 112, json!(true), vec![]).unwrap();
    assert_eq!(store.abandon(&retry), Err(StoreError::LeaseLost));
}

#[test]
fn failed_commit_changes_neither_receipt_nor_outbox() {
    let store = MemoryWorkflowStore::default();
    let lease = claim(&store, "one", "key", 100);
    let valid = event("one");
    assert!(store
        .commit(&lease, 101, json!(true), vec![valid.clone(), event("two")])
        .is_err());
    assert!(store.pending("one", 100).unwrap().is_empty());
    assert_eq!(
        store.claim("one", "key", "payload", 101, 10).unwrap(),
        Claim::InProgress
    );
    assert_eq!(
        store.commit(&lease, 101, json!(true), vec![valid.clone(), valid.clone()]),
        Err(StoreError::DuplicateEvent)
    );
    store
        .commit(&lease, 101, json!(true), vec![valid.clone()])
        .unwrap();
    store.mark_published("one", &valid.id).unwrap();
    let next = claim(&store, "one", "next", 100);
    assert_eq!(
        store.commit(&next, 101, json!(true), vec![valid]),
        Err(StoreError::DuplicateEvent)
    );
}

#[test]
fn simultaneous_claims_have_exactly_one_owner() {
    let store = Arc::new(MemoryWorkflowStore::default());
    let barrier = Arc::new(Barrier::new(8));
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let store = store.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                store.claim("tenant", "key", "payload", 100, 10).unwrap()
            })
        })
        .collect();
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Claim::Acquired(_)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Claim::InProgress))
            .count(),
        7
    );
}

#[test]
fn invalid_lease_parameters_are_rejected_and_instances_are_not_durable() {
    let store = MemoryWorkflowStore::default();
    assert!(store.claim("tenant", "key", "", 0, 10).is_err());
    assert!(store.claim("tenant", "key", "payload", 0, 0).is_err());
    assert!(store
        .claim("tenant", "key", "payload", u64::MAX, 10)
        .is_err());
    claim(&store, "tenant", "key", 0);
    let independent = MemoryWorkflowStore::default();
    claim(&independent, "tenant", "key", 0);
}
