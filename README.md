# `mlog` - Rust Logger

## Features
Supports varying log levels, colorized output, and optional file logging. 
  
## Log Level Colors
| Log Level |  Text Color  | Bg Color |
|-----------|--------------|----------|
| Info      | Blue         | None     |
| Warn      | Yellow       | None     |
| Error     | Red          | None     |
| Crit      | Bold White   | Red      |
| Success   | Bold White   | Green    |


## Missing Features
What's missing: currently designed for single-threaded program performance but lacks the ability to be used accross threads safely, & further optimization to make it fast


## Usage 

```rust
info!("This is an info message.");
warn!("This is a warning.");
error!("This is an error.");
crit!("Critical error, application will terminate.");
success!("Operation successful!");
````

## Initialization

You can initialize logger with any log level

```rust
init(LogLevel::Info, Some("logfile_name"));  // Creates "logfile_name.log"
````