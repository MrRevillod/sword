mod config;

use std::io;
use std::sync::OnceLock;

use tracing::Subscriber;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub use config::*;

static TRACING_INIT_STATUS: OnceLock<TracingInitStatus> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracingInitStatus {
    Disabled,
    Initialized,
    AlreadySet,
}

pub struct TracingSubscriber;

impl TracingSubscriber {
    #[clippy::allow(new_ret_no_self)]
    pub fn new(config: TracingConfig) -> Box<dyn Subscriber + Send + Sync> {
        let env_filter = if config.use_env_filter {
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(config.default_filter.clone()))
                .unwrap_or_else(|_| EnvFilter::new("info"))
        } else {
            EnvFilter::try_new(config.default_filter.clone())
                .unwrap_or_else(|_| EnvFilter::new("info"))
        };

        let builder = FmtSubscriber::builder()
            .with_env_filter(env_filter)
            .with_ansi(config.ansi)
            .with_target(config.target)
            .with_file(config.file)
            .with_line_number(config.line_number)
            .with_thread_ids(config.thread_ids)
            .with_thread_names(config.thread_names)
            .with_span_events(SpanEventsFmt::from(config.span_events).0);

        return match (config.output, config.format, config.timer) {
            (LogOutput::Stdout, LogFormat::Full, true) => {
                Box::new(builder.with_writer(io::stdout).finish())
            }
            (LogOutput::Stdout, LogFormat::Full, false) => {
                Box::new(builder.without_time().with_writer(io::stdout).finish())
            }
            (LogOutput::Stdout, LogFormat::Pretty, true) => {
                Box::new(builder.pretty().with_writer(io::stdout).finish())
            }
            (LogOutput::Stdout, LogFormat::Pretty, false) => Box::new(
                builder
                    .pretty()
                    .without_time()
                    .with_writer(io::stdout)
                    .finish(),
            ),
            (LogOutput::Stdout, LogFormat::Compact, true) => {
                Box::new(builder.compact().with_writer(io::stdout).finish())
            }
            (LogOutput::Stdout, LogFormat::Compact, false) => Box::new(
                builder
                    .compact()
                    .without_time()
                    .with_writer(io::stdout)
                    .finish(),
            ),
            (LogOutput::Stdout, LogFormat::Json, true) => {
                Box::new(builder.json().with_writer(io::stdout).finish())
            }
            (LogOutput::Stdout, LogFormat::Json, false) => Box::new(
                builder
                    .json()
                    .without_time()
                    .with_writer(io::stdout)
                    .finish(),
            ),
            (LogOutput::Stderr, LogFormat::Full, true) => {
                Box::new(builder.with_writer(io::stderr).finish())
            }
            (LogOutput::Stderr, LogFormat::Full, false) => {
                Box::new(builder.without_time().with_writer(io::stderr).finish())
            }
            (LogOutput::Stderr, LogFormat::Pretty, true) => {
                Box::new(builder.pretty().with_writer(io::stderr).finish())
            }
            (LogOutput::Stderr, LogFormat::Pretty, false) => Box::new(
                builder
                    .pretty()
                    .without_time()
                    .with_writer(io::stderr)
                    .finish(),
            ),
            (LogOutput::Stderr, LogFormat::Compact, true) => {
                Box::new(builder.compact().with_writer(io::stderr).finish())
            }
            (LogOutput::Stderr, LogFormat::Compact, false) => Box::new(
                builder
                    .compact()
                    .without_time()
                    .with_writer(io::stderr)
                    .finish(),
            ),
            (LogOutput::Stderr, LogFormat::Json, true) => {
                Box::new(builder.json().with_writer(io::stderr).finish())
            }
            (LogOutput::Stderr, LogFormat::Json, false) => Box::new(
                builder
                    .json()
                    .without_time()
                    .with_writer(io::stderr)
                    .finish(),
            ),
        };
    }

    pub fn init(config: TracingConfig) -> TracingInitStatus {
        if !config.enabled {
            return TracingInitStatus::Disabled;
        }

        let subscriber = Self::new(config);

        match tracing::subscriber::set_global_default(subscriber) {
            Ok(()) => TracingInitStatus::Initialized,
            Err(_) => TracingInitStatus::AlreadySet,
        }
    }

    pub fn init_once(config: TracingConfig) -> TracingInitStatus {
        *TRACING_INIT_STATUS.get_or_init(|| Self::init(config))
    }

    pub fn emit_init_status_log(status: TracingInitStatus, config: &TracingConfig) {
        match status {
            TracingInitStatus::Initialized => {
                tracing::info!(
                    target: "sword.startup.tracing",
                    format = ?config.format,
                    output = ?config.output,
                    default_filter = %config.default_filter,
                    use_env_filter = config.use_env_filter,
                    "Initialized Sword tracing subscriber"
                );

                tracing::trace!(
                    target: "sword.startup.tracing",
                    tracing_config = ?config,
                    "Resolved tracing configuration"
                );
            }
            TracingInitStatus::AlreadySet => {
                tracing::debug!(
                    target: "sword.startup.tracing",
                    "Tracing subscriber already set, using existing global subscriber"
                );
            }
            TracingInitStatus::Disabled => {
                if tracing::dispatcher::has_been_set() {
                    tracing::debug!(
                        target: "sword.startup.tracing",
                        "Sword tracing subscriber disabled in config, using existing global subscriber"
                    );
                }
            }
        }
    }
}
