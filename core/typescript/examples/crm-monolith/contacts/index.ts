import { pipeRules, ruleFailed, rulePassed, type Rule } from "../../../src/core/rules.js";
import type { MultiTenantCedarAuthorizer, OperationClaim } from "../../../src/core/authz.js";
import type { Claims } from "../../../src/core/auth.js";

export interface Contact {
  id: string;
  tenant_id: string;
  first_name: string;
  last_name: string;
  email: string;
  company: string;
  status: string;
}

export interface CreateContactInput {
  tenant_id: string;
  first_name: string;
  last_name: string;
  email: string;
  company: string;
}

export class InMemoryContactRepository {
  private storage = new Map<string, Contact>();

  async getById(tenantId: string, id: string): Promise<Contact | null> {
    return this.storage.get(`${tenantId}:${id}`) ?? null;
  }

  async getByEmail(tenantId: string, email: string): Promise<Contact | null> {
    for (const contact of this.storage.values()) {
      if (contact.tenant_id === tenantId && contact.email.toLowerCase() === email.toLowerCase()) {
        return contact;
      }
    }
    return null;
  }

  async save(contact: Contact): Promise<Contact> {
    this.storage.set(`${contact.tenant_id}:${contact.id}`, contact);
    return contact;
  }
}

// ─── Functional Business Rules (ARCH_TS_001 Compliant: No raw throws) ──────

export const validateContactEmailFormatRule: Rule<CreateContactInput> = (input) => {
  if (!input.email || !input.email.includes("@") || !input.email.includes(".")) {
    return ruleFailed("INVALID_EMAIL_FORMAT", `'${input.email}' is not a valid email address.`);
  }
  return rulePassed();
};

export function createEnsureContactEmailUniqueRule(repo: InMemoryContactRepository): Rule<CreateContactInput> {
  return async (input) => {
    const existing = await repo.getByEmail(input.tenant_id, input.email);
    if (existing) {
      return ruleFailed("DUPLICATE_CONTACT_EMAIL", `Contact with email '${input.email}' already exists in tenant '${input.tenant_id}'.`);
    }
    return rulePassed();
  };
}

// ─── Functional Pipeline & Handler ─────────────────────────────────────────

export async function createContact(
  input: CreateContactInput,
  repo: InMemoryContactRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Contact } | { ok: false; error_code: string; message: string }> {
  // 1. Cedar Authorization Check
  const claim: OperationClaim = {
    resource: "Contacts",
    action: "contacts.create",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  // 2. Functional Business Rule Pipeline
  const pipeline = pipeRules(
    validateContactEmailFormatRule,
    createEnsureContactEmailUniqueRule(repo)
  );
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  // 3. Persistence
  const contact: Contact = {
    id: Math.random().toString(36).substring(2, 10),
    tenant_id: input.tenant_id,
    first_name: input.first_name,
    last_name: input.last_name,
    email: input.email,
    company: input.company,
    status: "Lead",
  };

  const saved = await repo.save(contact);
  return { ok: true, value: saved };
}

export async function getContact(
  tenantId: string,
  contactId: string,
  repo: InMemoryContactRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Contact | null } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Contacts",
    action: "contacts.read",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const contact = await repo.getById(tenantId, contactId);
  return { ok: true, value: contact };
}
