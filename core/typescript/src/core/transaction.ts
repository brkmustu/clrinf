// @clrinf:generated — Transaction abstraction for the clrinf TypeScript runtime.

/**
 * Unit of Work interface for transactional command handling.
 * Implementations wrap database transactions (Prisma, Knex, TypeORM).
 */
export interface UnitOfWork {
  /** Begin a new transaction scope. */
  begin(): Promise<void>;

  /** Commit the current transaction. */
  commit(): Promise<void>;

  /** Rollback the current transaction. */
  rollback(): Promise<void>;
}

/**
 * Transaction middleware that wraps handler execution in a UoW scope.
 */
export async function withTransaction<T>(
  uow: UnitOfWork,
  handler: () => Promise<T>,
): Promise<T> {
  await uow.begin();
  try {
    const result = await handler();
    await uow.commit();
    return result;
  } catch (error) {
    await uow.rollback();
    throw error;
  }
}
