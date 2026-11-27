use async_trait::async_trait;
use clrinf_core::crud::{CrudEntity, CrudError, CrudQuery, CrudStore, PagedResult};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Mutex;

/// Durable SQLite implementation of [`CrudStore`] partitioned by `tenant_id`.
pub struct SqliteCrudStore<T: CrudEntity> {
    conn: Mutex<Connection>,
    table: String,
    _marker: PhantomData<T>,
}

fn validate_table_name(name: &str) -> Result<(), CrudError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(CrudError::Validation(
            "SQLite table name cannot be empty".into(),
        ));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(CrudError::Validation(format!(
            "Invalid SQLite table name '{name}'; expected [a-zA-Z_][a-zA-Z0-9_]*"
        )));
    }
    Ok(())
}

impl<T: CrudEntity> SqliteCrudStore<T> {
    /// Open (or create) a durable WAL-mode SQLite file for `table`.
    pub fn open(path: impl AsRef<Path>, table: &str) -> Result<Self, CrudError> {
        validate_table_name(table)?;
        if let Some(parent) = path.as_ref().parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| {
                CrudError::Storage(format!("Failed to create SQLite directory: {e}"))
            })?;
        }
        let conn = Connection::open(path.as_ref())
            .map_err(|e| CrudError::Storage(format!("SQLite open failed: {e}")))?;
        Self::init(conn, table)
    }

    /// Open an in-memory SQLite store (useful for fast isolated tests).
    pub fn in_memory(table: &str) -> Result<Self, CrudError> {
        validate_table_name(table)?;
        let conn = Connection::open_in_memory()
            .map_err(|e| CrudError::Storage(format!("SQLite open_in_memory failed: {e}")))?;
        Self::init(conn, table)
    }

    fn init(conn: Connection, table: &str) -> Result<Self, CrudError> {
        conn.execute_batch(&format!(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             CREATE TABLE IF NOT EXISTS {table} (
                 tenant_id TEXT NOT NULL,
                 id        TEXT NOT NULL,
                 payload   TEXT NOT NULL,
                 PRIMARY KEY (tenant_id, id)
             );"
        ))
        .map_err(|e| CrudError::Storage(format!("SQLite schema init failed: {e}")))?;

        Ok(Self {
            conn: Mutex::new(conn),
            table: table.to_string(),
            _marker: PhantomData,
        })
    }

    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CrudError> {
        self.conn
            .lock()
            .map_err(|_| CrudError::Storage("SQLite mutex poisoned".into()))
    }
}

#[async_trait]
impl<T: CrudEntity> CrudStore<T> for SqliteCrudStore<T> {
    async fn get_by_id(&self, tenant_id: &str, id: &T::Id) -> Result<Option<T>, CrudError> {
        let conn = self.lock_conn()?;
        let sql = format!(
            "SELECT payload FROM {} WHERE tenant_id = ?1 AND id = ?2",
            self.table
        );
        let raw: Option<String> = conn
            .query_row(&sql, params![tenant_id, id.to_string()], |row| row.get(0))
            .optional()
            .map_err(|e| CrudError::Storage(format!("SQLite get_by_id failed: {e}")))?;
        match raw {
            None => Ok(None),
            Some(json) => serde_json::from_str(&json)
                .map(Some)
                .map_err(|e| CrudError::Storage(format!("Invalid stored JSON payload: {e}"))),
        }
    }

    async fn list(&self, tenant_id: &str, query: &CrudQuery) -> Result<PagedResult<T>, CrudError> {
        let q = query.normalized();
        let conn = self.lock_conn()?;
        let offset = ((q.page - 1) * q.page_size) as i64;
        let limit = q.page_size as i64;

        let (total, payloads): (usize, Vec<String>) = if let Some(ref term) = q.search {
            let pattern = format!("%{}%", term.to_ascii_lowercase());
            let count_sql = format!(
                "SELECT COUNT(*) FROM {} WHERE tenant_id = ?1 AND LOWER(payload) LIKE ?2",
                self.table
            );
            let total: i64 = conn
                .query_row(&count_sql, params![tenant_id, &pattern], |row| row.get(0))
                .map_err(|e| CrudError::Storage(format!("SQLite count failed: {e}")))?;

            let list_sql = format!(
                "SELECT payload FROM {} WHERE tenant_id = ?1 AND LOWER(payload) LIKE ?2 ORDER BY id ASC LIMIT ?3 OFFSET ?4",
                self.table
            );
            let mut stmt = conn
                .prepare(&list_sql)
                .map_err(|e| CrudError::Storage(format!("SQLite prepare failed: {e}")))?;
            let rows = stmt
                .query_map(params![tenant_id, &pattern, limit, offset], |row| {
                    row.get::<_, String>(0)
                })
                .map_err(|e| CrudError::Storage(format!("SQLite list query failed: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CrudError::Storage(format!("SQLite row read failed: {e}")))?;
            (total.max(0) as usize, rows)
        } else {
            let count_sql = format!("SELECT COUNT(*) FROM {} WHERE tenant_id = ?1", self.table);
            let total: i64 = conn
                .query_row(&count_sql, params![tenant_id], |row| row.get(0))
                .map_err(|e| CrudError::Storage(format!("SQLite count failed: {e}")))?;

            let list_sql = format!(
                "SELECT payload FROM {} WHERE tenant_id = ?1 ORDER BY id ASC LIMIT ?2 OFFSET ?3",
                self.table
            );
            let mut stmt = conn
                .prepare(&list_sql)
                .map_err(|e| CrudError::Storage(format!("SQLite prepare failed: {e}")))?;
            let rows = stmt
                .query_map(params![tenant_id, limit, offset], |row| {
                    row.get::<_, String>(0)
                })
                .map_err(|e| CrudError::Storage(format!("SQLite list query failed: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CrudError::Storage(format!("SQLite row read failed: {e}")))?;
            (total.max(0) as usize, rows)
        };

        let mut items = Vec::with_capacity(payloads.len());
        for raw in payloads {
            let item: T = serde_json::from_str(&raw)
                .map_err(|e| CrudError::Storage(format!("Invalid stored JSON payload: {e}")))?;
            items.push(item);
        }

        Ok(PagedResult {
            items,
            total,
            page: q.page,
            page_size: q.page_size,
        })
    }

    async fn create(&self, tenant_id: &str, mut entity: T) -> Result<T, CrudError> {
        entity.set_tenant_id(tenant_id.to_string());
        let id = entity.id().to_string();
        let payload = serde_json::to_string(&entity)
            .map_err(|e| CrudError::Validation(format!("Failed to serialize entity: {e}")))?;
        let conn = self.lock_conn()?;
        let sql = format!(
            "INSERT INTO {} (tenant_id, id, payload) VALUES (?1, ?2, ?3)",
            self.table
        );
        match conn.execute(&sql, params![tenant_id, id, payload]) {
            Ok(_) => Ok(entity),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(CrudError::Conflict(format!(
                    "entity '{}' already exists",
                    entity.id()
                )))
            }
            Err(e) => Err(CrudError::Storage(format!("SQLite insert failed: {e}"))),
        }
    }

    async fn update(&self, tenant_id: &str, mut entity: T) -> Result<T, CrudError> {
        entity.set_tenant_id(tenant_id.to_string());
        let id = entity.id().to_string();
        let payload = serde_json::to_string(&entity)
            .map_err(|e| CrudError::Validation(format!("Failed to serialize entity: {e}")))?;
        let conn = self.lock_conn()?;
        let sql = format!(
            "UPDATE {} SET payload = ?3 WHERE tenant_id = ?1 AND id = ?2",
            self.table
        );
        let rows = conn
            .execute(&sql, params![tenant_id, id, payload])
            .map_err(|e| CrudError::Storage(format!("SQLite update failed: {e}")))?;
        if rows == 0 {
            return Err(CrudError::NotFound(format!(
                "entity '{}' not found",
                entity.id()
            )));
        }
        Ok(entity)
    }

    async fn delete(&self, tenant_id: &str, id: &T::Id) -> Result<bool, CrudError> {
        let conn = self.lock_conn()?;
        let sql = format!(
            "DELETE FROM {} WHERE tenant_id = ?1 AND id = ?2",
            self.table
        );
        let rows = conn
            .execute(&sql, params![tenant_id, id.to_string()])
            .map_err(|e| CrudError::Storage(format!("SQLite delete failed: {e}")))?;
        Ok(rows > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Widget {
        id: String,
        name: String,
        tenant_id: String,
    }

    impl CrudEntity for Widget {
        type Id = String;
        fn id(&self) -> Self::Id {
            self.id.clone()
        }
        fn set_id(&mut self, id: Self::Id) {
            self.id = id;
        }
        fn tenant_id(&self) -> &str {
            &self.tenant_id
        }
        fn set_tenant_id(&mut self, tenant_id: String) {
            self.tenant_id = tenant_id;
        }
    }

    #[tokio::test]
    async fn sqlite_store_crud_conflict_not_found_and_tenant_isolation() {
        let store = SqliteCrudStore::<Widget>::in_memory("widgets").unwrap();
        let item = Widget {
            id: "w-1".into(),
            name: "Gearbox".into(),
            tenant_id: String::new(),
        };
        let created = store.create("tenant-a", item.clone()).await.unwrap();
        assert_eq!(created.tenant_id, "tenant-a");

        // Duplicate create triggers Conflict
        let dup = store.create("tenant-a", item).await.unwrap_err();
        assert!(matches!(dup, CrudError::Conflict(_)));

        // Tenant isolation
        assert!(store
            .get_by_id("tenant-b", &"w-1".into())
            .await
            .unwrap()
            .is_none());
        assert_eq!(
            store
                .get_by_id("tenant-a", &"w-1".into())
                .await
                .unwrap()
                .unwrap()
                .name,
            "Gearbox"
        );

        // Missing update triggers NotFound
        let missing = store
            .update(
                "tenant-a",
                Widget {
                    id: "missing".into(),
                    name: "X".into(),
                    tenant_id: String::new(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(missing, CrudError::NotFound(_)));

        // Update & delete
        let upd = store
            .update(
                "tenant-a",
                Widget {
                    id: "w-1".into(),
                    name: "Pro Gearbox".into(),
                    tenant_id: String::new(),
                },
            )
            .await
            .unwrap();
        assert_eq!(upd.name, "Pro Gearbox");
        assert!(store.delete("tenant-a", &"w-1".into()).await.unwrap());
        assert!(!store.delete("tenant-a", &"w-1".into()).await.unwrap());
    }

    #[tokio::test]
    async fn sqlite_store_search_pagination_and_restart_durability() {
        let dir = std::env::temp_dir().join(format!("clrinf-sqlite-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let db_path = dir.join("crud.db");

        {
            let store = SqliteCrudStore::<Widget>::open(&db_path, "widgets").unwrap();
            for (id, name) in [("w-1", "Alpha"), ("w-2", "Beta Box"), ("w-3", "Gamma Box")] {
                store
                    .create(
                        "tenant-a",
                        Widget {
                            id: id.into(),
                            name: name.into(),
                            tenant_id: String::new(),
                        },
                    )
                    .await
                    .unwrap();
            }

            let searched = store
                .list(
                    "tenant-a",
                    &CrudQuery {
                        page: 1,
                        page_size: 1,
                        search: Some("box".into()),
                    },
                )
                .await
                .unwrap();
            assert_eq!(searched.total, 2);
            assert_eq!(searched.items.len(), 1);
            assert_eq!(searched.items[0].id, "w-2");
        }

        // Re-open DB and confirm durability across store drops
        let reopened = SqliteCrudStore::<Widget>::open(&db_path, "widgets").unwrap();
        let persisted = reopened
            .get_by_id("tenant-a", &"w-3".into())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(persisted.name, "Gamma Box");

        drop(reopened);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_unsafe_table_names() {
        assert!(validate_table_name("").is_err());
        assert!(validate_table_name("widgets; DROP TABLE x").is_err());
        assert!(validate_table_name("123bad").is_err());
        assert!(validate_table_name("valid_table_1").is_ok());
    }
}
