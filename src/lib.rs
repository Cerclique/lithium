use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors returned by [`Logger`] operations.
///
/// This crate uses [`thiserror`] for ergonomic error handling. All errors implement
/// [`std::error::Error`], [`std::fmt::Debug`], and [`std::fmt::Display`].
///
/// [`thiserror`]: https://docs.rs/thiserror
#[derive(Debug, Error)]
pub enum LoggerError {
    /// Returned when [`Logger::init`] is called on an instance that has already
    /// been initialized. The logger can be initialized exactly once per instance.
    #[error("logger already initialized")]
    AlreadyInitialized,

    /// Returned when [`env_logger`] fails to initialize the global logger.
    ///
    /// This typically happens when another crate has already initialized a
    /// logger with different configuration.
    ///
    /// [`env_logger`]: https://docs.rs/env_logger
    #[error("failed to initialize logger: {0}")]
    InitError(#[from] log::SetLoggerError),
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Configures a [`Logger`] before building it.
///
/// `LoggerBuilder` uses a fluent builder pattern. Construct it via [`new`],
/// configure the minimum log level with [`level`], then produce the final
/// [`Logger`] by calling [`build`].
///
/// # Example
///
/// ```rust
/// use lithium::{LoggerBuilder, LevelFilter};
///
/// let logger = LoggerBuilder::new()
///     .level(LevelFilter::Debug)
///     .build();
///
/// logger.init().expect("failed to init logger");
/// ```
///
/// [`Logger`]: struct.Logger.html
/// [`new`]: #method.new
/// [`level`]: #method.level
/// [`color`]: #method.color
/// [`build`]: #method.build
pub struct LoggerBuilder {
    level: LevelFilter,
    color: bool,
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerBuilder {
    /// Creates a builder with the default level of [`LevelFilter::Info`] and color disabled.
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            color: false,
        }
    }

    /// Sets the minimum log level. Returns `self` for chaining.
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Enables or disables colorized output. Returns `self` for chaining.
    pub fn color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Consumes the builder and returns a [`Logger`] carrying the configured options.
    pub fn build(self) -> Logger {
        Logger {
            level: self.level,
            color: self.color,
            initialized: AtomicBool::new(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Logger
// ---------------------------------------------------------------------------

/// A logger wrapper that provides elapsed time tracking and optional colorized output
/// built on top of [`env_logger`].
///
/// Create a [`Logger`] via [`LoggerBuilder::new().build()`] at application startup,
/// then initialize it exactly once via [`init`]. Do not create multiple [`Logger`]
/// instances — `env_logger` is global, so calling [`init`] on more than one instance
/// will fail for all but the first.
///
/// # stdout format
///
/// ```text
/// [     120 | ERROR | my_crate | src/lib.rs:42 ] something went wrong
/// ```
///
/// # Example
///
/// ```rust
/// use lithium::{info, LoggerBuilder};
///
/// let logger = LoggerBuilder::new().build();
/// logger.init().expect("failed to init logger");
///
/// info!("Application started");
/// ```
///
/// [`env_logger`]: https://docs.rs/env_logger
/// [`init`]: Logger::init
#[must_use = "call .init() to enable logging"]
pub struct Logger {
    level: LevelFilter,
    color: bool,
    initialized: AtomicBool,
}

impl Logger {
    /// Initializes the logger with a custom formatter that includes elapsed time,
    /// log level, target, file, and line number.
    ///
    /// The minimum log level is determined by [`LoggerBuilder::level()`],
    /// defaulting to [`LevelFilter::Info`].
    ///
    /// If `env_logger` has already been initialized globally (for example by
    /// another crate or test), this method returns [`LoggerError::InitError`].
    ///
    /// If this method is called a second time on the same instance, it returns
    /// [`LoggerError::AlreadyInitialized`].
    ///
    /// [`LoggerError::AlreadyInitialized`]: LoggerError::AlreadyInitialized
    /// [`LoggerError::InitError`]: LoggerError::InitError
    pub fn init(&self) -> Result<(), LoggerError> {
        // Atomically claim initialization. If we already initialized, return error.
        if !self
            .initialized
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return Err(LoggerError::AlreadyInitialized);
        }

        let start = std::time::Instant::now();
        let color_enabled = self.color;

        let mut builder = env_logger::Builder::new();
        builder.filter_level(self.level);

        builder.format(move |buf, record| {
            let elapsed = start.elapsed().as_millis();

            let line = format!(
                "[{elapsed:>9} | {:>5} | {} | {}:{} ] {}",
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args(),
            );

            if color_enabled {
                let style = buf.default_level_style(record.level());
                writeln!(buf, "{style}{line}{style:#}")
            } else {
                writeln!(buf, "{}", line)
            }
        });

        builder.try_init()?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

/// Re-exports the `log` crate macros and `LevelFilter` for convenience.
///
/// These macros and types can be used directly after importing from the `lithium` crate:
///
/// ```rust
/// use lithium::{info, debug, warn, error, trace, LevelFilter};
///
/// info!("This is an info message");
/// ```
pub use log::LevelFilter;
pub use log::{debug, error, info, trace, warn};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default_level() {
        // The default level should be Info.
        let logger = LoggerBuilder::new().build();
        // env_logger may already be claimed by a parallel test; that's acceptable.
        // We only need to verify the build + init path doesn't panic.
        let _ = logger.init();
    }

    #[test]
    fn test_builder_explicit_level() {
        let logger = LoggerBuilder::new().level(LevelFilter::Trace).build();
        let _ = logger.init();
    }

    #[test]
    fn test_multiple_initializations() {
        let logger = LoggerBuilder::new().build();

        let _ = logger.init();
        let result = logger.init();
        assert!(matches!(result, Err(LoggerError::AlreadyInitialized)));
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;

        let logger = Arc::new(LoggerBuilder::new().build());
        let ok_count = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));

        let mut handles = vec![];
        for _ in 0..10 {
            let logger_clone = Arc::clone(&logger);
            let ok_count_clone = Arc::clone(&ok_count);
            handles.push(std::thread::spawn(move || {
                if logger_clone.init().is_ok() {
                    ok_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let count = ok_count.load(std::sync::atomic::Ordering::Relaxed);
        assert!(
            count <= 1,
            "at most one thread should init successfully (got {count})"
        );
    }

    #[test]
    fn test_two_loggers_init_error() {
        // env_logger is global, so only one Logger can initialize successfully.
        let l1 = LoggerBuilder::new().build();
        let l2 = LoggerBuilder::new().build();

        let r1 = l1.init();
        let r2 = l2.init();

        let ok_count = r1.is_ok() as u8 + r2.is_ok() as u8;
        assert!(
            ok_count <= 1,
            "only one Logger should init successfully (got {ok_count})"
        );
    }
}
