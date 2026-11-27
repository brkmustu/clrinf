export interface Invoice {
  readonly invoiceId: string;
  readonly orderId: string;
  readonly amount: number;
}

export function validateInvoice(invoice: Invoice): { ok: true } | { ok: false; error: string } {
  if (invoice.amount <= 0) {
    return { ok: false, error: "Invoice amount must be positive" };
  }
  if (!invoice.orderId.trim()) {
    return { ok: false, error: "OrderId is required" };
  }
  return { ok: true };
}
