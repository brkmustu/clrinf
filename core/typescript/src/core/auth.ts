// @clrinf:generated — Authentication abstraction for the clrinf TypeScript runtime.

/**
 * Standard JWT claims carried through the pipeline.
 */
export interface Claims {
  sub: string;
  tenant_id: string;
  roles: string[];
  exp: number;
  iat: number;
}

/**
 * Token service interface for creating and verifying authentication tokens.
 */
export interface TokenService {
  /** Create a signed token for the given claims. */
  createToken(claims: Claims): Promise<string>;

  /** Verify and decode a token, returning the embedded claims. */
  verifyToken(token: string): Promise<Claims>;
}

/**
 * Authentication error.
 */
export class AuthenticationError extends Error {
  constructor(
    message: string,
    public readonly code: "INVALID_TOKEN" | "TOKEN_EXPIRED" | "MISSING_CREDENTIALS" = "INVALID_TOKEN",
  ) {
    super(message);
    this.name = "AuthenticationError";
  }
}
