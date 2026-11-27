use crate::domain::order::Order;

pub struct OrderPlacedV1<'a> {
    pub order_id: &'a str,
    pub customer_id: &'a str,
    pub total_amount: f64,
    pub currency: &'a str,
}

pub fn place_order(order: &Order) -> Result<(), &'static str> {
    order.validate()?;
    publishOrderPlacedV1(&OrderPlacedV1 {
        order_id: &order.id,
        customer_id: &order.customer_id,
        total_amount: order.total_cents() as f64,
        currency: &order.currency,
    });
    Ok(())
}

pub fn publishOrderPlacedV1(_event: &OrderPlacedV1<'_>) {
    // Delegates to generated CloudEvents outbox publisher
}

pub struct InvoiceIssuedV1Handler;
impl InvoiceIssuedV1Handler {
    pub fn handle(&self, order_id: &str) {
        let _ = order_id;
    }
}

pub struct StockReservedV1Handler;
impl StockReservedV1Handler {
    pub fn handle(&self, order_id: &str) {
        let _ = order_id;
    }
}
