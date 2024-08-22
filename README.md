# `mlog` - Rust Logger

## Features
Supports varying log levels, colorized output, optional file logging, async mode, multithreaded mode. 
  
## Log Level Colors
![Example Image](./tests/test-example.png)


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
shutdown(); // need to shutdown logger in order to ensure all log msgs are saved exiting.
````

## Configuration
```rust
let log_config = LogConfig {
    log_level: LogLevel::Info,                 // default to logging Everything
    program_name: "application".to_string(),  // default program name
    log_filepath: Some("logs/example"),      // filepath for logs (alt : None will just use console)
    console_flag: true,                     // toggle console logging
    async_flag: true,                     // async logging (default to false)
    multi_threaded_flag: true,           // single-threaded by default
    time_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),  // fully customizable time format
}
````