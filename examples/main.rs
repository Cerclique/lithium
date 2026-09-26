use lithium::{LevelFilter, LoggerBuilder, debug, error, info, trace, warn};

fn main() {
    let logger = LoggerBuilder::new()
        .level(LevelFilter::Debug)
        .color(true)
        .build();

    logger.init().expect("logger init failed");

    info!("Logger initialized successfully!");
    debug!("This is a debug message");
    trace!("This is a trace message");
    warn!("This is a warning message");
    error!("This is an error message");

    // Simulate some work to demonstrate elapsed time tracking
    std::thread::sleep(std::time::Duration::from_millis(150));
    info!("After a short delay");

    std::thread::sleep(std::time::Duration::from_millis(250));
    warn!("Another warning after more time has passed");

    // Calling init again returns an AlreadyInitialized error
    match logger.init() {
        Err(e) => println!("Expected init error: {e}"),
        Ok(()) => unreachable!("init should not succeed a second time"),
    }
}
