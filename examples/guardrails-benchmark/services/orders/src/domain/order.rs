pub struct OrderLine {
    pub sku: String,
    pub quantity: u32,
    pub unit_price_cents: u64,
}

pub struct Order {
    pub id: String,
    pub customer_id: String,
    pub currency: String,
    pub lines: Vec<OrderLine>,
}

impl Order {
    pub fn total_cents(&self) -> u64 {
        self.lines
            .iter()
            .map(|l| l.unit_price_cents * u64::from(l.quantity))
            .sum()
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.lines.is_empty() {
            return Err("Order must contain at least one line item");
        }
        if self.currency.len() != 3 {
            return Err("Currency must be a 3-letter ISO-4217 code");
        }
        Ok(())
    }
}
