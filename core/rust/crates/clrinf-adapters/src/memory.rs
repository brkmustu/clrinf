use clrinf_core::{
    context::validate_identifier,
    persistence::{Claim, IdempotencyStore, Lease, OutboxStore, StoreError, WorkflowStore},
    EventEnvelope, ValidationError,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Mutex,
};
use uuid::Uuid;

#[derive(Default)]
struct State {
    inbox: HashMap<(String, String), Entry>,
    outbox: BTreeMap<(String, String), EventEnvelope>,
    published: HashSet<(String, String)>,
}

struct Entry {
    fingerprint: String,
    token: String,
    expires_at: u64,
    response: Option<Value>,
}

/// Process-local, unbounded, non-durable storage. A mutex makes claim/commit atomic only
/// within this instance; no business writes or external delivery participate in that lock.
#[derive(Default)]
pub struct MemoryWorkflowStore {
    state: Mutex<State>,
}

impl MemoryWorkflowStore {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, State>, StoreError> {
        self.state
            .lock()
            .map_err(|_| StoreError::Unavailable("memory store mutex poisoned".into()))
    }
}

impl IdempotencyStore for MemoryWorkflowStore {
    fn claim(
        &self,
        tenant: &str,
        key: &str,
        fingerprint: &str,
        now: u64,
        lease_seconds: u64,
    ) -> Result<Claim, StoreError> {
        validate_identifier(tenant, "tenant")?;
        validate_identifier(key, "idempotency key")?;
        if fingerprint.is_empty() || lease_seconds == 0 {
            return Err(
                ValidationError("fingerprint and positive lease duration required".into()).into(),
            );
        }
        let expires_at = now
            .checked_add(lease_seconds)
            .ok_or_else(|| StoreError::Invalid(ValidationError("lease expiry overflow".into())))?;
        let mut state = self.lock()?;
        let identity = (tenant.to_owned(), key.to_owned());
        if let Some(entry) = state.inbox.get(&identity) {
            if entry.fingerprint != fingerprint {
                return Ok(Claim::Conflict);
            }
            if let Some(response) = &entry.response {
                return Ok(Claim::Replay(response.clone()));
            }
            if entry.expires_at > now {
                return Ok(Claim::InProgress);
            }
        }
        let token = Uuid::new_v4().to_string();
        state.inbox.insert(
            identity,
            Entry {
                fingerprint: fingerprint.into(),
                token: token.clone(),
                expires_at,
                response: None,
            },
        );
        Ok(Claim::Acquired(Lease {
            tenant_id: tenant.into(),
            key: key.into(),
            token,
        }))
    }

    fn abandon(&self, lease: &Lease) -> Result<(), StoreError> {
        let mut state = self.lock()?;
        let key = (lease.tenant_id.clone(), lease.key.clone());
        let entry = state.inbox.get(&key).ok_or(StoreError::LeaseLost)?;
        if entry.token != lease.token || entry.response.is_some() {
            return Err(StoreError::LeaseLost);
        }
        state.inbox.remove(&key);
        Ok(())
    }
}

impl WorkflowStore for MemoryWorkflowStore {
    fn commit(
        &self,
        lease: &Lease,
        now: u64,
        response: Value,
        events: Vec<EventEnvelope>,
    ) -> Result<(), StoreError> {
        let mut state = self.lock()?;
        let key = (lease.tenant_id.clone(), lease.key.clone());
        let entry = state.inbox.get(&key).ok_or(StoreError::LeaseLost)?;
        if entry.token != lease.token || entry.expires_at <= now || entry.response.is_some() {
            return Err(StoreError::LeaseLost);
        }
        let mut batch = HashSet::new();
        for event in &events {
            event.validate()?;
            if event.tenantid != lease.tenant_id {
                return Err(
                    ValidationError("outbox tenant differs from lease tenant".into()).into(),
                );
            }
            let identity = (event.tenantid.clone(), event.id.clone());
            if state.outbox.contains_key(&identity) || !batch.insert(identity) {
                return Err(StoreError::DuplicateEvent);
            }
        }
        // Every fallible check occurs before changing either collection.
        state
            .inbox
            .get_mut(&key)
            .ok_or(StoreError::LeaseLost)?
            .response = Some(response);
        for event in events {
            state
                .outbox
                .insert((event.tenantid.clone(), event.id.clone()), event);
        }
        Ok(())
    }
}

impl OutboxStore for MemoryWorkflowStore {
    fn pending(&self, tenant: &str, limit: usize) -> Result<Vec<EventEnvelope>, StoreError> {
        validate_identifier(tenant, "tenant")?;
        let state = self.lock()?;
        Ok(state
            .outbox
            .iter()
            .filter(|(key, _)| key.0 == tenant && !state.published.contains(*key))
            .take(limit)
            .map(|(_, event)| event.clone())
            .collect())
    }

    fn mark_published(&self, tenant: &str, event_id: &str) -> Result<(), StoreError> {
        validate_identifier(tenant, "tenant")?;
        validate_identifier(event_id, "event_id")?;
        let mut state = self.lock()?;
        let key = (tenant.into(), event_id.into());
        if !state.outbox.contains_key(&key) {
            return Err(StoreError::NotFound);
        }
        state.published.insert(key);
        Ok(())
    }
}
