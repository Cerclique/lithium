# Lithium

A logger wrapper with elapsed time tracking and colorized output built on [env_logger], [log], and [thiserror].

## Features

- Elapsed time tracking per log entry
- Custom format: `[elapsed_ms | LEVEL | target | file:line ] message`
- Programmatic log level configuration via [`LoggerBuilder`]
- Colorized output toggle via [`color`], disabled by default

## Usage

Create a [`LoggerBuilder`] and call [`build`] to produce a [`Logger`],
then call [`init`] once:

```rust
use lithium::{LoggerBuilder, LevelFilter};

fn main() {
    let logger = LoggerBuilder::new()
        .level(LevelFilter::Debug)
        .build();

    logger.init().expect("failed to init logger");
}
```

The default minimum level is `Info`. Override it with [`level`].
Colorized output is disabled by default. Enable it with [`color`].

Do not create multiple [`Logger`] instances or call [`init`] more than once.
`env_logger` is global, so subsequent calls will fail.

[`LoggerBuilder`]: https://docs.rs/lithium/latest/lithium/struct.LoggerBuilder.html
[`build`]: https://docs.rs/lithium/latest/lithium/struct.LoggerBuilder.html#method.build
[`init`]: https://docs.rs/lithium/latest/lithium/struct.Logger.html#method.init
[`level`]: https://docs.rs/lithium/latest/lithium/struct.LoggerBuilder.html#method.level
[`color`]: https://docs.rs/lithium/latest/lithium/struct.LoggerBuilder.html#method.color

## Log Levels

The following log levels are available, listed from least to most severe:

| Level   | Description                                      |
| ------- | ------------------------------------------------ |
| `Trace` | Finest-grained diagnostic events                 |
| `Debug` | Debugging information                            |
| `Info`  | General informational messages (default minimum) |
| `Warn`  | Warning conditions                               |
| `Error` | Critical errors and failures                     |

Set the minimum level via `LoggerBuilder::level(LevelFilter::Level)`. Messages at the minimum severity or higher are emitted.

## Format

### stdout

```text
[     120 | ERROR | my_crate | src/lib.rs:42 ] something went wrong
```

## License

[MIT](./LICENSE)

[env_logger]: https://docs.rs/env_logger
[log]: https://docs.rs/log
[thiserror]: https://docs.rs/thiserror
