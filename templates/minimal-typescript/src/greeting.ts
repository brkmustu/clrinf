import type { ApplicationModule, RequestContext, Result } from "./core.js";

export interface Greet {
  readonly name: string;
}

export interface Greeting {
  readonly message: string;
  readonly correlationId: string;
}

export class GreetingModule implements ApplicationModule<Greet, Greeting> {
  handle(request: Greet, context: RequestContext): Result<Greeting> {
    const name = request.name.trim();
    if (name.length === 0) {
      return {
        ok: false,
        error: {
          code: "validation.name_required",
          message: "Name must not be blank.",
        },
      };
    }

    return {
      ok: true,
      value: {
        message: `Hello, ${name}!`,
        correlationId: context.correlationId,
      },
    };
  }
}
