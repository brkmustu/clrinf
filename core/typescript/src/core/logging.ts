// @clrinf:generated — Logging abstraction for the clrinf TypeScript runtime.

/**
 * Structured log level.
 */
export type LogLevel = "trace" | "debug" | "info" | "warn" | "error";

/**
 * A structured log entry produced by the logging middleware.
 */
export interface LogEntry {
  level: LogLevel;
  message: string;
  target: string;
  fields: Record<string, unknown>;
  timestamp: string;
}

/**
 * Logger interface for structured logging.
 * Implementations can delegate to pino, console, or OpenTelemetry exporters.
 */
export interface Logger {
  log(entry: LogEntry): void;
  isEnabled(level: LogLevel): boolean;

  trace(message: string, fields?: Record<string, unknown>): void;
  debug(message: string, fields?: Record<string, unknown>): void;
  info(message: string, fields?: Record<string, unknown>): void;
  warn(message: string, fields?: Record<string, unknown>): void;
  error(message: string, fields?: Record<string, unknown>): void;
}

/**
 * Console-based structured logger (default implementation).
 */
export class ConsoleLogger implements Logger {
  private readonly minLevel: LogLevel;
  private static readonly LEVEL_ORDER: Record<LogLevel, number> = {
    trace: 0,
    debug: 1,
    info: 2,
    warn: 3,
    error: 4,
  };

  constructor(options: { minLevel?: LogLevel } = {}) {
    this.minLevel = options.minLevel ?? "info";
  }

  log(entry: LogEntry): void {
    if (!this.isEnabled(entry.level)) return;
    const out = `[${entry.timestamp}] [${entry.level.toUpperCase()}] ${entry.target}: ${entry.message}`;
    const fields = Object.keys(entry.fields).length > 0 ? ` ${JSON.stringify(entry.fields)}` : "";
    switch (entry.level) {
      case "error":
        console.error(out + fields);
        break;
      case "warn":
        console.warn(out + fields);
        break;
      default:
        console.log(out + fields);
    }
  }

  isEnabled(level: LogLevel): boolean {
    return ConsoleLogger.LEVEL_ORDER[level] >= ConsoleLogger.LEVEL_ORDER[this.minLevel];
  }

  trace(message: string, fields: Record<string, unknown> = {}): void {
    this.log({ level: "trace", message, target: "", fields, timestamp: new Date().toISOString() });
  }
  debug(message: string, fields: Record<string, unknown> = {}): void {
    this.log({ level: "debug", message, target: "", fields, timestamp: new Date().toISOString() });
  }
  info(message: string, fields: Record<string, unknown> = {}): void {
    this.log({ level: "info", message, target: "", fields, timestamp: new Date().toISOString() });
  }
  warn(message: string, fields: Record<string, unknown> = {}): void {
    this.log({ level: "warn", message, target: "", fields, timestamp: new Date().toISOString() });
  }
  error(message: string, fields: Record<string, unknown> = {}): void {
    this.log({ level: "error", message, target: "", fields, timestamp: new Date().toISOString() });
  }
}
