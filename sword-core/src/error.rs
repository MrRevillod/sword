use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum StartupPhase {
    Config,
    DI,
    Runtime,
    HttpAdapter,
    SocketIoAdapter,
    Interceptor,
    MacroInvariant,
}

impl Display for StartupPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let phase = match self {
            Self::Config => "CONFIG",
            Self::DI => "DI",
            Self::Runtime => "RUNTIME",
            Self::HttpAdapter => "HTTP_ADAPTER",
            Self::SocketIoAdapter => "SOCKETIO_ADAPTER",
            Self::Interceptor => "INTERCEPTOR",
            Self::MacroInvariant => "MACRO_INVARIANT",
        };

        write!(f, "{phase}")
    }
}

#[derive(Debug, Clone)]
pub struct StartupDiagnostic {
    _phase: StartupPhase,
    title: String,
    reason: String,
    context: Vec<(String, String)>,
    hints: Vec<String>,
}

#[macro_export]
macro_rules! sword_error {
    (
        phase: $phase:expr,
        title: $title:expr,
        reason: $reason:expr
        $(, context: { $($context_key:expr => $context_value:expr),* $(,)? })?
        $(, hints: [ $($hint:expr),* $(,)? ])?
        $(,)?
    ) => {{
        let mut diagnostic =
            $crate::error::StartupDiagnostic::new($phase, $title, ($reason).to_string());

        $(
            $(
                diagnostic = diagnostic.with_context(
                    $context_key,
                    ($context_value).to_string(),
                );
            )*
        )?

        $(
            $(
                diagnostic = diagnostic.with_hint(($hint).to_string());
            )*
        )?

        $crate::error::emit_fatal(diagnostic)
    }};
}

impl StartupDiagnostic {
    pub fn new(
        phase: StartupPhase,
        title: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            _phase: phase,
            title: title.into(),
            reason: reason.into(),
            context: Vec::new(),
            hints: Vec::new(),
        }
    }

    pub fn with_context(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }
}

impl Display for StartupDiagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nError: {}", self.title)?;

        let mut reason_lines = self.reason.lines();
        if let Some(first_line) = reason_lines.next() {
            write!(f, "\n\n↳ reason: {}", first_line.trim_start())?;

            for line in reason_lines {
                write!(f, "\n  {}", line.trim_start())?;
            }
        } else {
            write!(f, "\n\n↳ reason: <unknown>")?;
        }

        if !self.context.is_empty() {
            write!(f, "\n\n↳ context:")?;

            for (key, value) in &self.context {
                write!(f, "\n  - {key}: {value}")?;
            }
        }

        if !self.hints.is_empty() {
            write!(f, "\n\n↳ hints:")?;

            for hint in &self.hints {
                write!(f, "\n  - {hint}")?;
            }
        }

        Ok(())
    }
}

pub fn emit_fatal(diagnostic: StartupDiagnostic) -> ! {
    eprintln!("{diagnostic}");
    std::process::exit(1);
}
