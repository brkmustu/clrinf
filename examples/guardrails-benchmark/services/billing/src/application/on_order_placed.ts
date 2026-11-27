import { validateInvoice, type Invoice } from "../domain/invoice";

export interface OrderPlacedV1 {
  order_id: string;
  total_amount: number;
}

export class OrderPlacedV1Handler {
  async handle(payload: OrderPlacedV1): Promise<void> {
    const invoice: Invoice = {
      invoiceId: `inv_${payload.order_id}`,
      orderId: payload.order_id,
      amount: payload.total_amount,
    };
    const check = validateInvoice(invoice);
    if (!check.ok) {
      throw new Error(check.error);
    }
    publishInvoiceIssuedV1(invoice.invoiceId, invoice.orderId, invoice.amount);
  }
}

export function publishInvoiceIssuedV1(
  _invoiceId: string,
  _orderId: string,
  _amount: number,
): void {
  // Delegates to generated CloudEvents outbox publisher
}
