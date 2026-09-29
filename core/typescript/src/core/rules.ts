import { createError, type Context, type ErrorEnvelope, type JsonObject } from "./contracts";
import { Result } from "./result";

export interface RuleViolation {
  readonly errorCode: string;
  readonly message: string;
  readonly details?: JsonObject;
}

export type Rule<TCtx> = (
  context: TCtx,
  requestContext?: Context
) => Result<void, RuleViolation> | Promise<Result<void, RuleViolation>>;

export const rulePassed = (): Result<void, never> => Result.ok(undefined);

export const ruleFailed = (
  errorCode: string,
  message: string,
  details?: JsonObject
): Result<never, RuleViolation> =>
  Result.err({
    errorCode,
    message,
    ...(details !== undefined ? { details } : {}),
  });

/**
 * Composes multiple rules into a single sequential evaluation pipeline.
 * Short-circuits immediately upon the first failed rule.
 * Eliminates the need for raw 'throw' by enforcing Result<void, RuleViolation>.
 */
export function pipeRules<TCtx>(...rules: Rule<TCtx>[]): Rule<TCtx> {
  return async (context: TCtx, requestContext?: Context): Promise<Result<void, RuleViolation>> => {
    for (const rule of rules) {
      const result = await rule(context, requestContext);
      if (result.isErr) {
        return result;
      }
    }
    return rulePassed();
  };
}

/**
 * Converts a RuleViolation into a canonical, transport-independent ErrorEnvelope.
 */
export function violationToErrorEnvelope(
  violation: RuleViolation,
  requestContext: Context,
  retryable = false
): ErrorEnvelope {
  return createError(requestContext, {
    error_code: violation.errorCode,
    message: violation.message,
    retryable,
    ...(violation.details !== undefined ? { details: violation.details } : {}),
  });
}
