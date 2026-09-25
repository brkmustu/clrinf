export interface RequestContext {
  readonly correlationId: string;
}

export interface ApplicationError {
  readonly code: string;
  readonly message: string;
}

export type Result<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: ApplicationError };

export interface ApplicationModule<Request, Response> {
  handle(request: Request, context: RequestContext): Result<Response>;
}

export function dispatch<Request, Response>(
  module: ApplicationModule<Request, Response>,
  request: Request,
  context: RequestContext,
): Result<Response> {
  return module.handle(request, context);
}
