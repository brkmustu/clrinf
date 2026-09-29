// @clrinf:generated — Cache abstraction for the clrinf TypeScript runtime.

/**
 * Cache service interface for application-level caching.
 * Implementations: MemoryCache, Redis, etc.
 */
export interface CacheService {
  /** Store a value with an optional TTL (milliseconds). */
  set<T>(key: string, value: T, ttlMs?: number): Promise<void>;

  /** Retrieve a value. Returns undefined on cache miss. */
  get<T>(key: string): Promise<T | undefined>;

  /** Remove a single key. */
  remove(key: string): Promise<void>;

  /** Remove all keys matching the given prefix. */
  removeByPrefix(prefix: string): Promise<void>;
}

/**
 * In-memory cache implementation using Map.
 * Suitable for development and single-process deployments.
 */
export class MemoryCacheService implements CacheService {
  private readonly entries = new Map<string, { value: unknown; expiresAt: number }>();
  private readonly defaultTtlMs: number;
  private readonly now: () => number;

  constructor(options: { defaultTtlMs?: number; now?: () => number } = {}) {
    this.defaultTtlMs = options.defaultTtlMs ?? 300_000;
    this.now = options.now ?? Date.now;
  }

  async set<T>(key: string, value: T, ttlMs?: number): Promise<void> {
    this.evict();
    this.entries.set(key, {
      value,
      expiresAt: this.now() + (ttlMs ?? this.defaultTtlMs),
    });
  }

  async get<T>(key: string): Promise<T | undefined> {
    const entry = this.entries.get(key);
    if (!entry) return undefined;
    if (entry.expiresAt <= this.now()) {
      this.entries.delete(key);
      return undefined;
    }
    return entry.value as T;
  }

  async remove(key: string): Promise<void> {
    this.entries.delete(key);
  }

  async removeByPrefix(prefix: string): Promise<void> {
    for (const key of this.entries.keys()) {
      if (key.startsWith(prefix)) this.entries.delete(key);
    }
  }

  private evict(): void {
    const now = this.now();
    for (const [key, entry] of this.entries) {
      if (entry.expiresAt <= now) this.entries.delete(key);
    }
  }
}
