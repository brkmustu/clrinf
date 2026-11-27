use crate::AppError;
use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::MethodRouter,
    Extension, Json, Router,
};
use clrinf_core::{
    crud::{
        CrudEntity, CrudError, CrudId, CrudMetadata, CrudQuery, CrudRuleRunner, CrudStore,
        PagedResult,
    },
    RequestContext,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

/// Process-local in-memory [`CrudStore`] partitioned strictly by `tenant_id`.
pub struct InMemoryCrudStore<T: CrudEntity> {
    items: RwLock<HashMap<(String, String), T>>,
}

impl<T: CrudEntity> Default for InMemoryCrudStore<T> {
    fn default() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
        }
    }
}

impl<T: CrudEntity> InMemoryCrudStore<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl<T: CrudEntity> CrudStore<T> for InMemoryCrudStore<T> {
    async fn get_by_id(&self, tenant_id: &str, id: &T::Id) -> Result<Option<T>, CrudError> {
        let guard = self
            .items
            .read()
            .map_err(|_| CrudError::Storage("store lock poisoned".into()))?;
        Ok(guard.get(&(tenant_id.to_string(), id.to_string())).cloned())
    }

    async fn list(&self, tenant_id: &str, query: &CrudQuery) -> Result<PagedResult<T>, CrudError> {
        let q = query.normalized();
        let needle = q.search.as_ref().map(|s| s.to_ascii_lowercase());
        let guard = self
            .items
            .read()
            .map_err(|_| CrudError::Storage("store lock poisoned".into()))?;

        let mut matching: Vec<(String, T)> = guard
            .iter()
            .filter(|((t, _), _)| t == tenant_id)
            .filter(|(_, item)| match &needle {
                None => true,
                Some(n) => serde_json::to_string(item)
                    .map(|json| json.to_ascii_lowercase().contains(n))
                    .unwrap_or(false),
            })
            .map(|((_, id_key), item)| (id_key.clone(), item.clone()))
            .collect();

        matching.sort_by(|a, b| a.0.cmp(&b.0));
        let total = matching.len();
        let start = (q.page - 1).saturating_mul(q.page_size);
        let items = matching
            .into_iter()
            .skip(start)
            .take(q.page_size)
            .map(|(_, item)| item)
            .collect();

        Ok(PagedResult {
            items,
            total,
            page: q.page,
            page_size: q.page_size,
        })
    }

    async fn create(&self, tenant_id: &str, mut entity: T) -> Result<T, CrudError> {
        entity.set_tenant_id(tenant_id.to_string());
        let key = (tenant_id.to_string(), entity.id().to_string());
        let mut guard = self
            .items
            .write()
            .map_err(|_| CrudError::Storage("store lock poisoned".into()))?;
        if guard.contains_key(&key) {
            return Err(CrudError::Conflict(format!(
                "entity '{}' already exists",
                entity.id()
            )));
        }
        guard.insert(key, entity.clone());
        Ok(entity)
    }

    async fn update(&self, tenant_id: &str, mut entity: T) -> Result<T, CrudError> {
        entity.set_tenant_id(tenant_id.to_string());
        let key = (tenant_id.to_string(), entity.id().to_string());
        let mut guard = self
            .items
            .write()
            .map_err(|_| CrudError::Storage("store lock poisoned".into()))?;
        if !guard.contains_key(&key) {
            return Err(CrudError::NotFound(format!(
                "entity '{}' not found",
                entity.id()
            )));
        }
        guard.insert(key, entity.clone());
        Ok(entity)
    }

    async fn delete(&self, tenant_id: &str, id: &T::Id) -> Result<bool, CrudError> {
        let mut guard = self
            .items
            .write()
            .map_err(|_| CrudError::Storage("store lock poisoned".into()))?;
        Ok(guard
            .remove(&(tenant_id.to_string(), id.to_string()))
            .is_some())
    }
}

pub struct CrudState<T: CrudEntity, S: CrudStore<T>> {
    pub store: Arc<S>,
    pub rules: Arc<CrudRuleRunner<T>>,
}

impl<T: CrudEntity, S: CrudStore<T>> Clone for CrudState<T, S> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            rules: Arc::clone(&self.rules),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct CrudListParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
}

fn extract_context(ext: Option<Extension<RequestContext>>) -> Result<RequestContext, AppError> {
    match ext {
        Some(Extension(ctx)) => Ok(ctx),
        None => RequestContext::new("default", "crud", "crud").map_err(|e| AppError::Internal(e.0)),
    }
}

fn map_crud_error(err: CrudError) -> AppError {
    match err {
        CrudError::NotFound(m) => AppError::NotFound(m),
        CrudError::Conflict(m) => AppError::Conflict(m),
        CrudError::Validation(m) => AppError::BadRequest(m),
        CrudError::RuleViolation(v) => {
            AppError::BadRequest(format!("[{}] {}", v.error_code, v.message))
        }
        CrudError::Storage(m) => AppError::Internal(m),
    }
}

async fn create_handler<T: CrudMetadata, S: CrudStore<T>>(
    State(state): State<CrudState<T, S>>,
    ext: Option<Extension<RequestContext>>,
    Json(dto): Json<T::CreateDto>,
) -> Result<(StatusCode, Json<T>), AppError> {
    let ctx = extract_context(ext)?;
    let id = T::Id::generate_id();
    let entity = T::from_create(id, ctx.tenant_id.clone(), dto);
    state
        .rules
        .run_before_create(&entity, &ctx)
        .await
        .map_err(CrudError::from)
        .map_err(map_crud_error)?;
    let created = state
        .store
        .create(&ctx.tenant_id, entity)
        .await
        .map_err(map_crud_error)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn list_handler<T: CrudMetadata, S: CrudStore<T>>(
    State(state): State<CrudState<T, S>>,
    ext: Option<Extension<RequestContext>>,
    Query(params): Query<CrudListParams>,
) -> Result<Json<PagedResult<T>>, AppError> {
    let ctx = extract_context(ext)?;
    let query = CrudQuery {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        search: params.search,
    };
    let page = state
        .store
        .list(&ctx.tenant_id, &query)
        .await
        .map_err(map_crud_error)?;
    Ok(Json(page))
}

async fn get_by_id_handler<T: CrudMetadata, S: CrudStore<T>>(
    State(state): State<CrudState<T, S>>,
    ext: Option<Extension<RequestContext>>,
    Path(raw_id): Path<String>,
) -> Result<Json<T>, AppError> {
    let ctx = extract_context(ext)?;
    let id = T::Id::from_str(&raw_id)
        .map_err(|_| AppError::BadRequest(format!("Invalid {} ID '{}'", T::ENTITY_NAME, raw_id)))?;
    let found = state
        .store
        .get_by_id(&ctx.tenant_id, &id)
        .await
        .map_err(map_crud_error)?
        .ok_or_else(|| AppError::NotFound(format!("{} '{}' not found", T::ENTITY_NAME, raw_id)))?;
    Ok(Json(found))
}

async fn update_handler<T: CrudMetadata, S: CrudStore<T>>(
    State(state): State<CrudState<T, S>>,
    ext: Option<Extension<RequestContext>>,
    Path(raw_id): Path<String>,
    Json(dto): Json<T::UpdateDto>,
) -> Result<Json<T>, AppError> {
    let ctx = extract_context(ext)?;
    let id = T::Id::from_str(&raw_id)
        .map_err(|_| AppError::BadRequest(format!("Invalid {} ID '{}'", T::ENTITY_NAME, raw_id)))?;
    let existing = state
        .store
        .get_by_id(&ctx.tenant_id, &id)
        .await
        .map_err(map_crud_error)?
        .ok_or_else(|| AppError::NotFound(format!("{} '{}' not found", T::ENTITY_NAME, raw_id)))?;
    let mut proposed = existing.clone();
    proposed.apply_update(dto);
    state
        .rules
        .run_before_update(&existing, &proposed, &ctx)
        .await
        .map_err(CrudError::from)
        .map_err(map_crud_error)?;
    let updated = state
        .store
        .update(&ctx.tenant_id, proposed)
        .await
        .map_err(map_crud_error)?;
    Ok(Json(updated))
}

async fn delete_handler<T: CrudMetadata, S: CrudStore<T>>(
    State(state): State<CrudState<T, S>>,
    ext: Option<Extension<RequestContext>>,
    Path(raw_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let ctx = extract_context(ext)?;
    let id = T::Id::from_str(&raw_id)
        .map_err(|_| AppError::BadRequest(format!("Invalid {} ID '{}'", T::ENTITY_NAME, raw_id)))?;
    let existing = state
        .store
        .get_by_id(&ctx.tenant_id, &id)
        .await
        .map_err(map_crud_error)?
        .ok_or_else(|| AppError::NotFound(format!("{} '{}' not found", T::ENTITY_NAME, raw_id)))?;
    state
        .rules
        .run_before_delete(&existing, &ctx)
        .await
        .map_err(CrudError::from)
        .map_err(map_crud_error)?;
    state
        .store
        .delete(&ctx.tenant_id, &id)
        .await
        .map_err(map_crud_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Mounts Axum routes at `T::ROUTE_PREFIX` and `T::ROUTE_PREFIX/:id` for every
/// operation enabled in `T::OPERATIONS`.
pub fn crud_router<T: CrudMetadata, S: CrudStore<T>>(
    store: Arc<S>,
    rules: Arc<CrudRuleRunner<T>>,
) -> Router {
    let state = CrudState { store, rules };
    let ops = T::OPERATIONS;

    let mut collection = MethodRouter::new();
    let mut has_collection = false;
    if ops.contains(&"create") {
        collection = collection.post(create_handler::<T, S>);
        has_collection = true;
    }
    if ops.contains(&"get_list") {
        collection = collection.get(list_handler::<T, S>);
        has_collection = true;
    }

    let mut item = MethodRouter::new();
    let mut has_item = false;
    if ops.contains(&"get_by_id") {
        item = item.get(get_by_id_handler::<T, S>);
        has_item = true;
    }
    if ops.contains(&"update") {
        item = item.put(update_handler::<T, S>);
        has_item = true;
    }
    if ops.contains(&"delete") {
        item = item.delete(delete_handler::<T, S>);
        has_item = true;
    }

    let mut router: Router<CrudState<T, S>> = Router::new();
    if has_collection {
        router = router.route(T::ROUTE_PREFIX, collection);
    }
    if has_item {
        let item_path = format!("{}/:id", T::ROUTE_PREFIX.trim_end_matches('/'));
        router = router.route(&item_path, item);
    }
    router.with_state(state)
}
