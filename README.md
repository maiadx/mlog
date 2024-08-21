# Rust Logger

Supports varying log levels, colorized output, and optional file logging. 
The logger is designed for single-threaded usage for the moment but will include multithreaded support soon

## Features

- **Log Levels**: `Info`, `Warn`, `Error`, `Crit`, `Success`

  
## Log Level Colors (may only work on unix systems)
| Log Level | Text Color          | Background Color |
|-----------|---------------------|------------------|
| Info      | Blue                | None             |
| Warn      | Yellow              | None             |
| Error     | Red                 | None             |
| Crit      | Bold White          | Red              |
| Success   | Bold White          | Green            |

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