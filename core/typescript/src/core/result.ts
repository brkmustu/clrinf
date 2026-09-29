/**
 * A lightweight, zero-dependency, idiomatic Result type for functional error handling.
 * Enforces explicit error propagation instead of throwing raw exceptions.
 */

export type Result<T, E> = Ok<T> | Err<E>;

export class Ok<T> {
  readonly isOk = true as const;
  readonly isErr = false as const;

  constructor(readonly value: T) {}

  map<U>(fn: (val: T) => U): Result<U, never> {
    return new Ok(fn(this.value));
  }

  mapErr<F>(_fn: (err: never) => F): Result<T, F> {
    return this;
  }

  unwrapOr(_defaultValue: T): T {
    return this.value;
  }

  match<U>(patterns: { ok: (val: T) => U; err: (err: never) => U }): U {
    return patterns.ok(this.value);
  }
}

export class Err<E> {
  readonly isOk = false as const;
  readonly isErr = true as const;

  constructor(readonly error: E) {}

  map<U>(_fn: (val: never) => U): Result<U, E> {
    return this;
  }

  mapErr<F>(fn: (err: E) => F): Result<never, F> {
    return new Err(fn(this.error));
  }

  unwrapOr<T>(defaultValue: T): T {
    return defaultValue;
  }

  match<U>(patterns: { ok: (val: never) => U; err: (err: E) => U }): U {
    return patterns.err(this.error);
  }
}

export const Result = {
  ok<T>(value: T): Ok<T> {
    return new Ok(value);
  },

  err<E>(error: E): Err<E> {
    return new Err(error);
  },

  fromThrowable<T, E = Error>(fn: () => T, transformErr?: (e: unknown) => E): Result<T, E> {
    try {
      return Result.ok(fn());
    } catch (err) {
      return Result.err(transformErr ? transformErr(err) : (err as E));
    }
  },

  async fromAsyncThrowable<T, E = Error>(
    fn: () => Promise<T>,
    transformErr?: (e: unknown) => E
  ): Promise<Result<T, E>> {
    try {
      const val = await fn();
      return Result.ok(val);
    } catch (err) {
      return Result.err(transformErr ? transformErr(err) : (err as E));
    }
  },
};
