use serde::{Deserialize, Serialize};
use thisconfig::ConfigItem;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TracingConfig {
    /// Enables or disables global tracing initialization.
    ///
    /// When `false`, `TracingSubscriber::init` and `init_once` return without
    /// registering a subscriber.
    pub enabled: bool,
    /// Reads filter directives from the `RUST_LOG` environment variable.
    ///
    /// If enabled, the environment value is attempted first. If parsing fails
    /// or the variable is missing, `default_filter` is used as fallback.
    #[serde(rename = "use-env-filter")]
    pub use_env_filter: bool,
    /// Fallback filter directives for `tracing_subscriber::EnvFilter`.
    ///
    /// Accepts standard tracing directives like `info`, `warn,my_crate=debug`,
    /// etc. This value is always used when `use_env_filter` is `false`.
    #[serde(rename = "default-filter")]
    pub default_filter: String,
    /// Event formatting style used by the subscriber output.
    ///
    /// `full` follows tracing-subscriber defaults, while `pretty`, `compact`,
    /// and `json` provide alternative renderers.
    pub format: LogFormat,
    /// Output stream where logs are written.
    ///
    /// Choose `stdout` for regular output pipelines or `stderr` to keep logs
    /// separated from application stdout data.
    pub output: LogOutput,
    /// Enables ANSI color/style escape sequences in text formats.
    ///
    /// This only affects human-readable outputs (`full`, `pretty`, `compact`)
    /// and may be ignored by terminals or CI environments.
    pub ansi: bool,
    /// Includes the event target (module path) in each record.
    ///
    /// Useful to identify which crate/module emitted a log event when multiple
    /// components share the same runtime.
    pub target: bool,
    /// Includes source file information for each event.
    ///
    /// Adds compile-time file metadata to logs when available.
    pub file: bool,
    /// Includes source line number for each event.
    ///
    /// Typically used together with `file` for precise source location.
    #[serde(rename = "line-number")]
    pub line_number: bool,
    /// Includes the current thread ID in each event.
    ///
    /// Helpful for diagnosing concurrency issues in multi-threaded runtimes.
    #[serde(rename = "thread-ids")]
    pub thread_ids: bool,
    /// Includes the current thread name in each event.
    ///
    /// Useful when worker threads are explicitly named.
    #[serde(rename = "thread-names")]
    pub thread_names: bool,
    /// Enables timestamp emission in formatter output.
    ///
    /// When disabled, logs are emitted without time metadata.
    pub timer: bool,
    /// Controls which span lifecycle events are emitted as log entries.
    ///
    /// A **span** represents a unit of work with a start and an end
    /// (for example: `handle_request`, `query_db`, `process_payment`).
    ///
    /// Unlike a regular `tracing` event (a single point-in-time log line),
    /// span events describe lifecycle transitions of that unit of work:
    /// created, entered, exited, and closed.
    ///
    /// This setting decides which of those transitions are printed by the
    /// formatter. It maps directly to
    /// `tracing_subscriber::fmt::format::FmtSpan`.
    ///
    /// Tip for beginners:
    /// - Start with `none` to keep logs quiet.
    /// - Use `new`/`close` to see operation boundaries.
    /// - Use `active`/`full` when debugging nested async/concurrent flows.
    #[serde(rename = "span-events")]
    pub span_events: SpanEvents,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Full,
    Pretty,
    Compact,
    Json,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    #[default]
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SpanEvents {
    /// Do not emit synthetic lifecycle logs for spans.
    ///
    /// Only regular `tracing::event!` / `tracing::info!` / `tracing::error!`
    /// logs appear.
    #[default]
    None,
    /// Emit one log when a span is created.
    ///
    /// Useful to see when a unit of work starts.
    New,
    /// Emit one log when a span is closed (dropped).
    ///
    /// Useful to see when a unit of work ends.
    Close,
    /// Emit logs on each enter and exit of a span.
    ///
    /// Helpful to visualize nesting and execution flow.
    Active,
    /// Emit all lifecycle logs: create, enter, exit, close.
    ///
    /// Most verbose option; useful for deep debugging.
    Full,
}

pub struct SpanEventsFmt(pub FmtSpan);

impl From<SpanEvents> for SpanEventsFmt {
    fn from(mode: SpanEvents) -> Self {
        let value = match mode {
            SpanEvents::None => FmtSpan::NONE,
            SpanEvents::New => FmtSpan::NEW,
            SpanEvents::Close => FmtSpan::CLOSE,
            SpanEvents::Active => FmtSpan::ACTIVE,
            SpanEvents::Full => FmtSpan::FULL,
        };

        Self(value)
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_env_filter: true,
            default_filter: "info".to_string(),
            format: LogFormat::Full,
            output: LogOutput::Stderr,
            ansi: true,
            target: true,
            file: false,
            line_number: false,
            thread_ids: false,
            thread_names: false,
            timer: true,
            span_events: SpanEvents::None,
        }
    }
}

impl ConfigItem for TracingConfig {
    fn key() -> &'static str {
        "tracing"
    }
}
