export type TaxRegion = "EU" | "US" | "TR";

export function calculateTax(amount: number, region: TaxRegion): number {
  const rate = region === "TR" ? 0.2 : region === "EU" ? 0.19 : 0.08;
  return Math.round(amount * rate * 100) / 100;
}

export function formatCurrency(amount: number, currency: string): string {
  return `${currency.toUpperCase()} ${amount.toFixed(2)}`;
}
