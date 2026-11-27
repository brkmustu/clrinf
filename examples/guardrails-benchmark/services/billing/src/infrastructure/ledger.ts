export interface BillingLedgerStore {
  record(invoiceId: string, amount: number): Promise<void>;
}

export function createInMemoryLedger(): BillingLedgerStore {
  const rows = new Map<string, number>();
  return {
    async record(invoiceId: string, amount: number) {
      rows.set(invoiceId, amount);
    },
  };
}
