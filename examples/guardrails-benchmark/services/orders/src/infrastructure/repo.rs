pub trait OrderRepository {
    fn save(&self, order_id: &str, payload: &str) -> Result<(), String>;
    fn find_by_id(&self, order_id: &str) -> Option<String>;
}

pub fn health_check() -> &'static str {
    "orders-ok"
}
