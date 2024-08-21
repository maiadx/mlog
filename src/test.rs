use std::sync::atomic::{AtomicPtr, Ordering};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::ptr;
use chrono::Local;
use std::cell::UnsafeCell;


// Console colors for log types
pub const CONSOLE_COLOR_RED: &str = "\x1b[1;31m";
pub const CONSOLE_COLOR_GREEN: &str = "\x1b[32m";
pub const CONSOLE_COLOR_YELLOW: &str = "\x1b[01;33m";
pub const CONSOLE_COLOR_BLUE: &str = "\x1b[34m";
pub const CONSOLE_COLOR_MAGENTA: &str = "\x1b[1;35m";
pub const CONSOLE_COLOR_CYAN: &str = "\x1b[0;36m";
pub const CONSOLE_COLOR_BOLD_WHITE: &str = "\x1b[97m";
pub const CONSOLE_COLOR_RESET: &str = "\x1b[0m";

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Crit, // for critial errors - specifically those that will likely terminate the program, pretty self-explanatory :)
    Success,
}

// Logger configuration for file sink
struct LoggerConfig {
    file: Option<UnsafeCell<Option<BufWriter<File>>>>,  // Optional buffered file logging
}

impl LoggerConfig {
    fn new(log_to_file: Option<&str>) -> Self {
        let file = log_to_file.map(|path| {
            let file = OpenOptions::new().append(true).create(true).open(path).unwrap();
            UnsafeCell::new(Some(BufWriter::new(file)))
        });
        LoggerConfig { file }
    }
}

// Global logger state
#[allow(dead_code)]
struct Logger {
    level: LogLevel,   // Controls which log levels are allowed
    config: LoggerConfig,  // Config for file sink
}
#[allow(dead_code)]
impl Logger {
    fn new(level: LogLevel, log_to_file: Option<&str>) -> Self {
        Logger {
            level, 
            config: LoggerConfig::new(log_to_file),  // Pass file sink option to LoggerConfig
        }
    }

    fn log(&self, level: LogLevel, message: &str, color: &str) {
        if level >= self.level {
            let time = Local::now();
            let formatted_message = format!(
                "{}[{}] - {:?} - {}{}", 
                color, 
                time.format("%Y-%m-%d %H:%M:%S%.3f"), 
                level, 
                message, 
                CONSOLE_COLOR_RESET
            );

            // Console output with color
            println!("{}", formatted_message);

            // Optional file output, using config.file
            if let Some(file_cell) = self.config.file.as_ref() {
                unsafe {
                    if let Some(ref mut file) = *file_cell.get() {
                        writeln!(file, "{}", formatted_message).unwrap();
                    }
                }
            }
        }
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, CONSOLE_COLOR_BLUE);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, CONSOLE_COLOR_YELLOW);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, CONSOLE_COLOR_RED);
    }

    pub fn critical(&self, message: &str) {
        self.log(LogLevel::Crit, message, CONSOLE_COLOR_MAGENTA);
    }

    pub fn success(&self, message: &str) {
        self.log(LogLevel::Success, message, CONSOLE_COLOR_GREEN);
    }
}

// Global static logger using AtomicPtr
static LOGGER: AtomicPtr<Logger> = AtomicPtr::new(ptr::null_mut());

/// Function to initialize the logger with optional file output
pub fn init(level: LogLevel, log_to_file: Option<&str>) {
    let logger = Box::new(Logger::new(level, log_to_file));
    let logger_ptr = Box::into_raw(logger);
    
    let old_logger = LOGGER.swap(logger_ptr, Ordering::SeqCst);
    
    // If there was an old logger, deallocate it
    if !old_logger.is_null() {
        unsafe {
            let _ = Box::from_raw(old_logger);
        }
    }
}
#[allow(dead_code)]
fn with_logger<F: FnOnce(&Logger)>(f: F) {
    let logger_ptr = LOGGER.load(Ordering::SeqCst);
    if !logger_ptr.is_null() {
        unsafe {
            f(&*logger_ptr);
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        with_logger(|logger| logger.info(&format!($($arg)*)));
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        with_logger(|logger| logger.warn(&format!($($arg)*)));
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        with_logger(|logger| logger.error(&format!($($arg)*)));
    }};
}

#[macro_export]
macro_rules! crit {
    ($($arg:tt)*) => {{
        with_logger(|logger| logger.crit(&format!($($arg)*)));
    }};
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        with_logger(|logger| logger.success(&format!($($arg)*)));
    }};        // Initialize logger with Info level and optional file sink

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging() {

        // lib initialization :
        init(LogLevel::Info, Some("test-app"));

        info!("This is an info message.");
        warn!("This is a warning message.");
        error!("This is an error message.");
        critical!("This is a critical message.");
        success!("This is a success message.");
    }
}