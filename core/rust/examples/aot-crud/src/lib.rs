pub mod domain;
pub mod rules;

use axum::{middleware, Router};
use clrinf_adapters::{context_middleware, crud_router, InMemoryCrudStore};
use clrinf_core::crud::CrudRuleRunner;
use domain::{Brand, Category};
use rules::ActiveProductsCheck;
use std::sync::Arc;

pub fn build_app() -> Router {
    let category_store = Arc::new(InMemoryCrudStore::<Category>::new());
    let category_rules = Arc::new(
        CrudRuleRunner::new().with_before_delete(ActiveProductsCheck),
    );

    let brand_store = Arc::new(InMemoryCrudStore::<Brand>::new());
    let brand_rules = Arc::new(CrudRuleRunner::new());

    Router::new()
        .merge(crud_router::<Category, _>(category_store, category_rules))
        .merge(crud_router::<Brand, _>(brand_store, brand_rules))
        .layer(middleware::from_fn(context_middleware))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use clrinf_macros::ClrinfCrud;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[derive(Debug, Clone, Serialize, Deserialize, ClrinfCrud)]
    #[clrinf_crud(route = "/api/notes", skip(update))]
    struct Note {
        pub id: String,
        pub title: String,
        #[crud_ignore]
        pub tenant_id: String,
    }

    fn req_with_tenant(method: &str, uri: &str, tenant: &str, body: Option<Value>) -> Request<Body> {
        let builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-tenant-id", tenant)
            .header("x-correlation-id", "corr-1")
            .header("x-causation-id", "caus-1");
        match body {
            Some(val) => builder
                .header("content-type", "application/json")
                .body(Body::from(val.to_string()))
                .unwrap(),
            None => builder.body(Body::empty()).unwrap(),
        }
    }

    #[tokio::test]
    async fn category_and_brand_crud_enforces_tenant_and_delete_rule() {
        let app = build_app();

        // 1. Create Category in tenant-a
        let res = app
            .clone()
            .oneshot(req_with_tenant(
                "POST",
                "/api/categories",
                "tenant-a",
                Some(json!({ "name": "Books", "slug": "books" })),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let created: Value = serde_json::from_slice(&to_bytes(res.into_body(), 16384).await.unwrap()).unwrap();
        let id = created["id"].as_str().unwrap().to_string();
        assert_eq!(created["tenant_id"], "tenant-a");

        // 2. Tenant isolation: tenant-b cannot see tenant-a's category
        let res_other = app
            .clone()
            .oneshot(req_with_tenant(
                "GET",
                &format!("/api/categories/{id}"),
                "tenant-b",
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res_other.status(), StatusCode::NOT_FOUND);

        // 3. Update Category in tenant-a
        let res_upd = app
            .clone()
            .oneshot(req_with_tenant(
                "PUT",
                &format!("/api/categories/{id}"),
                "tenant-a",
                Some(json!({ "name": "Tech Books", "slug": "protected" })),
            ))
            .await
            .unwrap();
        assert_eq!(res_upd.status(), StatusCode::OK);

        // 4. BeforeDeleteRule rejects deleting 'protected'
        let res_del_blocked = app
            .clone()
            .oneshot(req_with_tenant(
                "DELETE",
                &format!("/api/categories/{id}"),
                "tenant-a",
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res_del_blocked.status(), StatusCode::BAD_REQUEST);
        let err_body: Value = serde_json::from_slice(&to_bytes(res_del_blocked.into_body(), 16384).await.unwrap()).unwrap();
        assert!(err_body["message"].as_str().unwrap().contains("CATEGORY_PROTECTED"));

        // 5. Create and list Brand
        let res_brand = app
            .clone()
            .oneshot(req_with_tenant(
                "POST",
                "/api/brands",
                "tenant-a",
                Some(json!({ "name": "Acme" })),
            ))
            .await
            .unwrap();
        assert_eq!(res_brand.status(), StatusCode::CREATED);

        let res_list = app
            .oneshot(req_with_tenant("GET", "/api/brands", "tenant-a", None))
            .await
            .unwrap();
        assert_eq!(res_list.status(), StatusCode::OK);
        let list_json: Value = serde_json::from_slice(&to_bytes(res_list.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(list_json["total"], 1);
    }

    #[tokio::test]
    async fn skip_attribute_omits_ejected_operation_route() {
        let store = Arc::new(InMemoryCrudStore::<Note>::new());
        let app = crud_router::<Note, _>(store, Arc::new(CrudRuleRunner::new()))
            .layer(middleware::from_fn(context_middleware));

        let res_create = app
            .clone()
            .oneshot(req_with_tenant(
                "POST",
                "/api/notes",
                "tenant-a",
                Some(json!({ "title": "First" })),
            ))
            .await
            .unwrap();
        assert_eq!(res_create.status(), StatusCode::CREATED);
        let created: Value = serde_json::from_slice(&to_bytes(res_create.into_body(), 16384).await.unwrap()).unwrap();
        let id = created["id"].as_str().unwrap();

        let res_put = app
            .oneshot(req_with_tenant(
                "PUT",
                &format!("/api/notes/{id}"),
                "tenant-a",
                Some(json!({ "title": "Updated" })),
            ))
            .await
            .unwrap();
        assert_eq!(res_put.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
