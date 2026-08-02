use env_logger::Env;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

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

/// A logger wrapper that provides elapsed time tracking and colorized output
/// built on top of [`env_logger`].
///
/// Create a single [`Logger`] instance at application startup and initialize
/// it exactly once via [`init`]. Do not create multiple [`Logger`] instances —
/// `env_logger` is global, so calling [`init`] on more than one instance will
/// fail for all but the first.
///
/// The built-in formatter emits log lines in the shape:
///
/// ```text
/// [elapsed_ms | LEVEL | target | file:line ] message
/// ```
///
/// For example:
///
/// ```text
/// [     120 | ERROR | my_crate | src/lib.rs:42 ] something went wrong
/// ```
///
/// # Example
///
/// ```rust
/// use lithium::{info, Logger};
///
/// let logger = Logger::new();
/// logger.init().expect("failed to init logger");
///
/// info!("Application started");
/// ```
///
/// [`env_logger`]: https://docs.rs/env_logger
/// [`init`]: Logger::init
#[must_use = "call .init() to enable logging"]
pub struct Logger {
    initialized: AtomicBool,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            initialized: AtomicBool::new(false),
        }
    }
}

impl Logger {
    /// Creates a new, uninitialized [`Logger`] instance.
    ///
    /// No logging is active until [`init`][Logger::init] is called.
    ///
    /// # Panics
    ///
    /// This method never panics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes the logger with a custom formatter that includes elapsed time,
    /// log level, target, file, and line number.
    ///
    /// The default log filter is [`LevelFilter::Info`]. Override it by setting the
    /// `RUST_LOG` environment variable.
    ///
    /// If `env_logger` has already been initialized globally (for example by
    /// another crate or test), this method returns [`LoggerError::InitError`].
    ///
    /// If this method is called a second time on the same instance, it returns
    /// [`LoggerError::AlreadyInitialized`].
    ///
    /// [`LevelFilter::Info`]: log::LevelFilter::Info
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

        // We successfully claimed initialization, now set up env_logger.
        let start = std::time::Instant::now();

        let mut builder = env_logger::Builder::from_env(Env::default());
        builder
            .format(move |buf, record| {
                let style = buf.default_level_style(record.level());
                let elapsed = start.elapsed().as_millis();

                writeln!(
                    buf,
                    "{style}[{elapsed:>9} | {:>5} | {} | {}:{} ] {}{style:#}",
                    record.level(),
                    record.target(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args(),
                )
            })
            .try_init()?;

        Ok(())
    }
}

/// Re-exports the `log` crate macros for convenience.
///
/// These macros can be used directly after importing from the `logger` crate:
///
/// ```rust
/// use lithium::{info, debug, warn, error, trace};
///
/// info!("This is an info message");
/// ```
pub use log::{debug, error, info, trace, warn};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_initialization_ok() {
        let logger = Logger::new();
        assert!(logger.init().is_ok());
    }

    #[test]
    fn test_multiple_initializations() {
        let logger = Logger::new();

        let _ = logger.init();
        let result = logger.init();
        assert!(matches!(result, Err(LoggerError::AlreadyInitialized)));
    }

    #[test]
    fn test_thread_safety() {
        let logger = Arc::new(Logger::new());
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
        // We verify that calling init on two different Logger instances never
        // yields two Ok results.
        let l1 = Logger::new();
        let l2 = Logger::new();

        let r1 = l1.init();
        let r2 = l2.init();

        let ok_count = r1.is_ok() as u8 + r2.is_ok() as u8;
        assert!(
            ok_count <= 1,
            "only one Logger should init successfully (got {ok_count})"
        );
    }
}
