pub fn validate_discount_bps(bps: u32) -> Result<u32, &'static str> {
    if bps > 4_000 {
        return Err("Discount cannot exceed 40%");
    }
    Ok(bps)
}

pub fn apply_discount(subtotal_cents: u64, bps: u32) -> Result<u64, &'static str> {
    let valid_bps = validate_discount_bps(bps)?;
    let discount = subtotal_cents.saturating_mul(u64::from(valid_bps)) / 10_000;
    Ok(subtotal_cents.saturating_sub(discount))
}
