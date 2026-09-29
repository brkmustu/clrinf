// @clrinf:generated — Multi-Tenant Authorization abstraction for the clrinf TypeScript runtime.
//
// Provides policy-based access control with default multi-tenant isolation,
// Cedar-compatible ABAC/RBAC rule evaluation, and explicit forbid guarantees.

import type { Claims } from "./auth";

/**
 * An operation claim required for a specific action (e.g., "orders.create"),
 * carrying an optional resource tenant for cross-tenant isolation enforcement.
 */
export interface OperationClaim {
  resource: string;
  action: string;
  resource_tenant_id?: string;
}

/**
 * Detailed authorization decision with multi-tenant context.
 */
export type AuthzDecision =
  | { allowed: true; reason?: string }
  | { allowed: false; reason: string; is_tenant_violation?: boolean };

/**
 * Authorization service interface for evaluating access policies.
 */
export interface AuthorizationService {
  /** Check whether the given claims satisfy the required operation claims under multi-tenant rules. */
  authorize(claims: Claims, required: OperationClaim[]): Promise<AuthzDecision>;
}

/**
 * Authorization error.
 */
export class AuthorizationError extends Error {
  constructor(
    message: string,
    public readonly is_tenant_violation: boolean = false,
    public readonly resource?: string,
    public readonly action?: string,
  ) {
    super(message);
    this.name = "AuthorizationError";
  }
}

/**
 * Cedar-compatible policy rule representation.
 */
export interface CedarPolicyRule {
  effect_permit: boolean; // true = permit, false = forbid
  principal_role?: string;
  action?: string;
  resource_type?: string;
}

/**
 * Multi-Tenant Cedar Policy Engine.
 * Guarantees:
 * 1. Default Multi-Tenant Isolation: Cross-tenant access is strictly denied (Forbid),
 *    unless the principal has the PlatformAdmin role.
 * 2. Role-based permit checks aligned with Cedar policies.
 * 3. Cedar Forbid rules always override permit rules.
 */
export class MultiTenantCedarAuthorizer implements AuthorizationService {
  private rules: CedarPolicyRule[] = [];

  constructor(private readonly platformAdminRole: string = "PlatformAdmin") {}

  /** Register an additional Cedar policy rule. */
  addRule(rule: CedarPolicyRule): this {
    this.rules.push(rule);
    return this;
  }

  async authorize(claims: Claims, required: OperationClaim[]): Promise<AuthzDecision> {
    const isPlatformAdmin = claims.roles.includes(this.platformAdminRole);

    for (const op of required) {
      // 1. Multi-Tenant İzolasyon Kontrolü (Default Tenant Guard)
      if (op.resource_tenant_id && op.resource_tenant_id !== claims.tenant_id && !isPlatformAdmin) {
        return {
          allowed: false,
          is_tenant_violation: true,
          reason: `Tenant isolation violation: principal tenant '${claims.tenant_id}' cannot access resource tenant '${op.resource_tenant_id}'`,
        };
      }

      // PlatformAdmin tüm izin kurallarını bypass eder
      if (isPlatformAdmin) {
        continue;
      }

      // 2. Cedar Forbid Kuralları Kontrolü (Forbid her zaman Permit'i ezer)
      const forbidTriggered = this.rules
        .filter((r) => !r.effect_permit)
        .some((r) => {
          const roleMatch = !r.principal_role || claims.roles.includes(r.principal_role);
          const actionMatch = !r.action || r.action === op.action || r.action === "*";
          const resMatch = !r.resource_type || r.resource_type === op.resource || r.resource_type === "*";
          return roleMatch && actionMatch && resMatch;
        });

      if (forbidTriggered) {
        return {
          allowed: false,
          is_tenant_violation: false,
          reason: `Explicit Cedar FORBID triggered for ${op.resource}:${op.action}`,
        };
      }

      // 3. İzin Kontrolü (TenantAdmin, spesifik rol veya Claim)
      const isTenantAdmin = claims.roles.includes("TenantAdmin") || claims.roles.includes("Admin");
      const expectedPermission = `${op.resource}.${op.action}`;
      const hasRolePermission =
        claims.roles.includes(expectedPermission) || claims.roles.includes(op.resource);

      if (!isTenantAdmin && !hasRolePermission) {
        // Tanımlı Cedar permit kuralları kontrolü
        const permittedByRule = this.rules
          .filter((r) => r.effect_permit)
          .some((r) => {
            const roleMatch = !r.principal_role || claims.roles.includes(r.principal_role);
            const actionMatch = !r.action || r.action === op.action || r.action === "*";
            const resMatch = !r.resource_type || r.resource_type === op.resource || r.resource_type === "*";
            return roleMatch && actionMatch && resMatch;
          });

        if (!permittedByRule) {
          return {
            allowed: false,
            is_tenant_violation: false,
            reason: `Insufficient permissions for operation: ${expectedPermission}`,
          };
        }
      }
    }

    return { allowed: true };
  }
}
