// @clrinf:generated — Logging abstraction for the clrinf Rust runtime.
//
// Provides a structured logging trait that mirrors the logging pipeline
// behavior pattern. Implementations can delegate to `tracing`, `log`,
// or OpenTelemetry exporters.

use std::collections::HashMap;

/// Structured log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A structured log entry produced by the logging middleware.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub target: String,
    pub fields: HashMap<String, String>,
}

/// Logging port trait. Middleware injects structured log entries through this.
pub trait LogPort: Send + Sync + 'static {
    fn log(&self, entry: LogEntry);
    fn is_enabled(&self, level: LogLevel) -> bool;
}

/// Default tracing-compatible logger that delegates to the `tracing` crate macros.
pub struct TracingLogger;

impl LogPort for TracingLogger {
    fn log(&self, entry: LogEntry) {
        match entry.level {
            LogLevel::Trace => tracing::trace!(target = %entry.target, "{}", entry.message),
            LogLevel::Debug => tracing::debug!(target = %entry.target, "{}", entry.message),
            LogLevel::Info => tracing::info!(target = %entry.target, "{}", entry.message),
            LogLevel::Warn => tracing::warn!(target = %entry.target, "{}", entry.message),
            LogLevel::Error => tracing::error!(target = %entry.target, "{}", entry.message),
        }
    }

    fn is_enabled(&self, _level: LogLevel) -> bool {
        true // tracing subscriber handles filtering
    }
}
