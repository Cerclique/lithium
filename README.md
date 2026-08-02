# Lithium

A logger wrapper with elapsed time tracking and colorized output built on [env_logger], [log], and [thiserror].

## Features

- Colorized log output with elapsed time tracking
- Custom format: `[elapsed_ms | LEVEL | target | file:line ] message`
- `RUST_LOG` environment variable support for log filtering

## Usage

Create a single [`Logger`] and call [`init`] once:

```rust
use lithium::{Logger, info};

fn main() {
    let logger = Logger::new();
    logger.init().expect("failed to init logger");

    info!("Logger initialized!");
}
```

Do not create multiple [`Logger`] instances or call [`init`] more than once.
`env_logger` is global, so subsequent calls will fail.

[`Logger`]: https://docs.rs/logger/latest/lithium/struct.Logger.html
[`init`]: https://docs.rs/logger/latest/lithium/struct.Logger.html#method.init

## Format

```text
[     120 | ERROR | my_crate | src/lib.rs:42 ] something went wrong
```

## License

[MIT](./LICENSE)

[env_logger]: https://docs.rs/env_logger
[log]: https://docs.rs/log
[thiserror]: https://docs.rs/thiserror
