mod config;
mod dev;
mod time;

use std::io;
use std::sync::OnceLock;

use dev::DevFormatter;
use time::TimestampFormatter;
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
    pub fn new(config: TracingConfig) -> Box<dyn Subscriber + Send + Sync> {
        let env_filter = if config.use_env_filter {
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(config.default_filter.clone()))
                .unwrap_or_else(|_| EnvFilter::new("info"))
        } else {
            EnvFilter::try_new(config.default_filter.clone())
                .unwrap_or_else(|_| EnvFilter::new("info"))
        };

        let timer = TimestampFormatter::from_config(&config);
        let with_target = config.has_field(TracingField::Target);
        let with_file = config.has_field(TracingField::File);
        let with_line_number = config.has_field(TracingField::LineNumber);
        let with_thread_ids = config.has_field(TracingField::ThreadId);
        let with_thread_names = config.has_field(TracingField::ThreadName);

        let builder = FmtSubscriber::builder()
            .with_env_filter(env_filter)
            .with_ansi(config.ansi)
            .with_thread_ids(with_thread_ids)
            .with_thread_names(with_thread_names);

        macro_rules! finish_standard {
            ($builder:expr, $writer:expr) => {
                Box::new(
                    $builder
                        .with_timer(timer.clone())
                        .with_writer($writer)
                        .with_target(with_target)
                        .with_file(with_file)
                        .with_line_number(with_line_number)
                        .finish(),
                )
            };
        }

        macro_rules! finish_dev {
            ($writer:expr) => {
                Box::new(
                    builder
                        .event_format(DevFormatter::new(
                            config.ansi,
                            &config.with_fields,
                            timer.clone(),
                        ))
                        .with_writer($writer)
                        .finish(),
                )
            };
        }

        match (config.output, config.format) {
            (LogOutput::Stdout, LogFormat::Full) => {
                finish_standard!(builder, io::stdout)
            }
            (LogOutput::Stdout, LogFormat::Pretty) => {
                finish_standard!(builder.pretty(), io::stdout)
            }
            (LogOutput::Stdout, LogFormat::Compact) => {
                finish_standard!(builder.compact(), io::stdout)
            }
            (LogOutput::Stdout, LogFormat::Json) => {
                finish_standard!(builder.json(), io::stdout)
            }
            (LogOutput::Stdout, LogFormat::Dev) => finish_dev!(io::stdout),
            (LogOutput::Stderr, LogFormat::Full) => {
                finish_standard!(builder, io::stderr)
            }
            (LogOutput::Stderr, LogFormat::Pretty) => {
                finish_standard!(builder.pretty(), io::stderr)
            }
            (LogOutput::Stderr, LogFormat::Compact) => {
                finish_standard!(builder.compact(), io::stderr)
            }
            (LogOutput::Stderr, LogFormat::Json) => {
                finish_standard!(builder.json(), io::stderr)
            }
            (LogOutput::Stderr, LogFormat::Dev) => finish_dev!(io::stderr),
        }
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
                    "Initialized tracing subscriber"
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
                        "Tracing subscriber disabled in config, using existing global subscriber"
                    );
                }
            }
        }
    }
}
