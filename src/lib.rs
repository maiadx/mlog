use std::sync::atomic::{AtomicPtr, Ordering};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::ptr;
use chrono::Local;
use std::cell::UnsafeCell;
// use std::sync::Mutex;

/* 
 */

pub const CONSOLE_COLOR_RED: &str = "\x1b[1;31m";
pub const CONSOLE_COLOR_GREEN: &str = "\x1b[32m";
pub const CONSOLE_COLOR_YELLOW: &str = "\x1b[01;33m";
pub const CONSOLE_COLOR_BLUE: &str = "\x1b[34m";
pub const CONSOLE_COLOR_MAGENTA: &str = "\x1b[1;35m";
pub const CONSOLE_COLOR_CYAN: &str = "\x1b[0;36m";
pub const CONSOLE_COLOR_BOLD_WHITE: &str = "\x1b[97m";
pub const CONSOLE_COLOR_RESET: &str = "\x1b[0m";
pub const CONSOLE_BG_COLOR_RED: &str = "\x1b[41m";
pub const CONSOLE_BG_COLOR_GREEN: &str = "\x1b[42m";

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Crit, // critial errors - specifically those that will likely terminate the program, pretty self-explanatory :)
    Success,
}

// Logger configuration for opt. file sink
/*  using UnsafeCell<T> allows for mutability behind immutable ref - safe for single threaded programs which is what this is being used for at the moment
 */
    #[allow(dead_code)]
struct LogConfig {
    file: Option<UnsafeCell<Option<BufWriter<File>>>>  
}

#[allow(dead_code)]
impl LogConfig { 
    fn new(log_filepath: Option<&str>) -> Self {
        let file = log_filepath.map(|path| {
            // Automatically append ".log" if not already present
            let log_file_name = if path.ends_with(".log") {
                path.to_string()
            } else {
                format!("{}.log", path)
            };

            // check if file exists before opening
            // let file_exists: bool = std::path::Path::new(&log_file_name).exists();

            // open file
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&log_file_name)
                .unwrap();

            let mut writer = BufWriter::new(file);

            // session info
            writeln!(
                writer,
                "\n\n----------------------------------------------------------\n///////// Session Started at {} ///////// \n----------------------------------------------------------\n",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            )
            .unwrap();
            writer.flush().unwrap(); 
            

            UnsafeCell::new(Some(writer))
        });

        LogConfig { file }
    }
}

// pub enum LoggerType {
//     SingleThread,
//     MultiThread,
// }


#[allow(dead_code)]
pub struct Logger {
    level : LogLevel,
    config : LogConfig,
}

static LOGGER : AtomicPtr<Logger> = AtomicPtr::new(ptr::null_mut());


#[allow(dead_code)]
impl Logger { 
    fn new (level : LogLevel, log_filepath : Option<&str>) -> Self {
        Logger {level, config : LogConfig::new(log_filepath)}
    }

    fn log(&self, level: LogLevel, msg: &str, fg_color: &str, bg_color: &str) {
        if level >= self.level {
            let time = Local::now();
            let formatted_msg = format!(
                "{}{}[{}] - {:?} - {}{}",
                fg_color,
                bg_color,
                time.format("%H:%M:%S"),
                level,
                msg,
                CONSOLE_COLOR_RESET
            );
    
            // log to console
            println!("{}", formatted_msg);
    
            // log to file if a file is configured
            if let Some(file_cell) = self.config.file.as_ref() {
                unsafe {
                    if let Some(ref mut file) = *file_cell.get() {
                        // Write the formatted message to the file
                        if writeln!(file, "{}", formatted_msg).is_err() {
                            eprintln!("Failed to write log to file");
                        }
    
                        // Flush the buffer to ensure it is written to disk
                        if file.flush().is_err() {
                            eprintln!("Failed to flush the log file");
                        }
                    }
                }
            }
        }
    }

    pub fn info(&self, msg : &str) {
        self.log(LogLevel::Info, msg, CONSOLE_COLOR_BLUE, "");
    }
    pub fn warn(&self, msg : &str) {
        self.log(LogLevel::Warn, msg, CONSOLE_COLOR_YELLOW, "");
    }
    pub fn error(&self, msg : &str) {
        self.log(LogLevel::Error, msg, CONSOLE_COLOR_RED, "");
    }
    pub fn crit(&self, msg : &str) {
        self.log(LogLevel::Crit, msg, CONSOLE_COLOR_BOLD_WHITE,CONSOLE_BG_COLOR_RED);
    }
    pub fn success(&self, msg : &str) {
        self.log(LogLevel::Success, msg, CONSOLE_COLOR_BOLD_WHITE, CONSOLE_BG_COLOR_GREEN);
    }

            
} 

pub fn init(level: LogLevel, log_filepath: Option<&str>) {
    let logger_ptr = LOGGER.load(Ordering::SeqCst);

    // logger ptr  should be nullptr at start of initialization (only used once per program)
    if !logger_ptr.is_null() { 
        panic!("Logger is already initialized! Only one mlog instance is allowed.")
    }


    let logger = Box::new(Logger::new(level, log_filepath));
    let logger_ptr = Box::into_raw(logger);

    LOGGER.store(logger_ptr, Ordering::SeqCst);
}
    
#[allow(dead_code)]
pub fn with_logger<F: FnOnce(&Logger)>(f: F) {
    let logger_ptr = LOGGER.load(Ordering::SeqCst);
        unsafe {
            f(&*logger_ptr);
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
    }};
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging() {

        // example initialization :
        init(LogLevel::Info, Some("test-app"));

        info!("This is an info msg.");
        warn!("This is a warning msg.");
        error!("This is an error msg.");
        crit!("This is a critical msg.");
        success!("This is a successful msg :)");
    }
}