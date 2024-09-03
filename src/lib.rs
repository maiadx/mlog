use std::sync::atomic::{AtomicUsize, AtomicPtr, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{thread, ptr, fs};
use std::cell::UnsafeCell;
use std::time::Duration;
// use std::sync::mpsc::{self, Sender};
use std::{io::BufWriter, io::Write};
use chrono::Local;
use std::panic::{self};
use std::fs::{File, OpenOptions};

const BUFFER_CAPACITY: usize = 15;  
const MAX_LOG_FILE_SIZE: u64 = 10 * 1024 * 1024;  // 10 MB max log file size before rotation to new file

pub const CONSOLE_COLOR_WHITE: &str = "\x1b[37m";
pub const CONSOLE_COLOR_BLUE: &str = "\x1b[94m";
pub const CONSOLE_COLOR_YELLOW: &str = "\x1b[01;33m";
pub const CONSOLE_COLOR_PINK: &str = "\x1b[38;5;212m";
pub const CONSOLE_COLOR_RED: &str = "\x1b[1;31m";
pub const CONSOLE_BG_COLOR_RED: &str = "\x1b[41m";
pub const CONSOLE_BG_COLOR_GREEN: &str = "\x1b[42m";
pub const CONSOLE_COLOR_RESET: &str = "\x1b[0m";
pub const CONSOLE_COLOR_GREEN: &str = "\x1b[1;32m"; 
pub const CONSOLE_COLOR_MAGENTA: &str = "\x1b[35m";    


#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum LogLevel {
    Info = 0b11111,    // Everything
    Success = 0b01111, // Crit, Error, Warn, and Success messages
    Warn = 0b00111,    // Crit, Error, and Warn messages
    Error = 0b00011,   // Crit and Error messages
    Crit = 0b00001,    // Only Crit messages
}


pub struct LogConfig {
    pub log_level: LogLevel,
    pub application_name: String,
    pub log_filepath: Option<String>,   // Optional log file path
    pub console_flag: bool,            // Flag to log to console
    pub async_flag: bool,             // Flag to enable async logging
    pub multi_threaded_flag: bool,   // Flag for multi-threaded mode
    pub time_format: String,        // Time format string
}


impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            log_level: LogLevel::Info,                 // Default to logging Everything
            application_name: "default application".to_string(),  // Default program name
            log_filepath: None,                      // No log file by default
            console_flag: true,                     // Log to console by default
            async_flag: false,                     // No async by default
            multi_threaded_flag: false,           // Single-threaded by default
            time_format: "%Y-%m-%d %H:%M:%S".to_string(),  // Default time format with milliseconds
        }
    }
}


pub struct Logger {
    pub config: LogConfig,
    log_level_mask : u8,                                    // bitmask for log-levels
    buffer: [UnsafeCell<Option<String>>; BUFFER_CAPACITY], // Use UnsafeCell for interior mutability
    head: AtomicUsize,  // Atomic index for the head of the buffer (write position in async mode)
    tail: AtomicUsize,  // Atomic index for the tail of the buffer (read/flush position in async mode)
    should_run: Arc<AtomicBool>,  // Control flag for async thread
    flush_interval: Duration,
    mutex: Option<Mutex<()>>, // Mutex for thread-safe access when async mode is disabled
    file_writer: Option<Mutex<BufWriter<File>>>,  // Writer for log file
}

unsafe impl Sync for Logger {} // Required for sharing the logger across threads when async mode is enabled

impl Logger {
    pub fn new(config: LogConfig) -> Arc<Self> {
        // Only create the file writer if a valid log file path is provided
        let log_file = config.log_filepath.as_ref().map(|p| {
            let file_path = if p.ends_with(".log") {
                p.clone()
            } else {
                format!("{}.log", p)
            };

            // Create a log file and wrap it in a Mutex for safe access
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .expect("Failed to open log file");
            Mutex::new(BufWriter::new(file))
        });

        let tmp_async_flag = config.async_flag.clone();
        let tmp_mt_flag = config.multi_threaded_flag.clone();
        let tmp_log_level = config.log_level.clone();
        
        
        // Initialize the logger with the configuration
        let logger = Arc::new(Logger {
            config,
            log_level_mask: tmp_log_level as u8,
            buffer: Default::default(),  // Initialize buffer
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            should_run: Arc::new(AtomicBool::new(true)),
            flush_interval: Duration::from_secs(5),  // Default flush interval
            mutex: if tmp_mt_flag && !tmp_async_flag {
                Some(Mutex::new(()))
            } else {
                None
            },
            file_writer: log_file,  // Only set up file writer if file path is provided
        });

        // Log session start info if logging to a file
        if let Some(ref writer) = logger.file_writer {
            let mut writer_guard = writer.lock().unwrap();
            
            // Format the session start text
            let session_text = format!(
                "///////// {} : Session Started at {} /////////",
                logger.config.application_name,
                Local::now().format(&logger.config.time_format)
            );
        
            // Calculate the length of the `-` line based on the session text length
            let line_length = session_text.len();
            let separator_line = "-".repeat(line_length);
        
            // Write the session start with dynamic separator lines
            writeln!(
                writer_guard,
                "\n\n{}\n{}\n{}\n",
                separator_line, session_text, separator_line
            ).expect("Failed to write session start to log file");
            
            writer_guard.flush().expect("Failed to flush session start to log file");
        }
        

        // Spawn async flush thread if necessary
        if logger.config.async_flag {
            let logger_arc = Arc::clone(&logger);
            let should_run = Arc::clone(&logger_arc.should_run);
            thread::spawn(move || {
                while should_run.load(Ordering::Relaxed) {
                    thread::sleep(logger_arc.flush_interval);
                    logger_arc.flush(); // Periodic flush
                }
            });
        }
        logger
    }

    pub fn write_log(&self, log_msg: &str) {
        // Write to console if console_flag is enabled
        if self.config.console_flag {
            println!("{}", log_msg);
        }
    
        // Write to the log file if file_writer is available
        if let Some(ref file_writer) = self.file_writer {
            let mut file_writer = file_writer.lock().unwrap();
            writeln!(file_writer, "{}", log_msg).expect("Failed to write log to file");
            file_writer.flush().expect("Failed to flush log file");
        }
    }


    pub fn log(&self, level: LogLevel, msg: &str, color: &str) {

        if level as u8 > self.log_level_mask {
            return;  // Skip this log, as the level is higher than the configured mask
        }

        let time = Local::now();
        let formatted_msg = format!(
            "{}[{}] {} \x1b[0m",
            color,
            time.format(self.config.time_format.as_str()),
            msg
        );

        if self.config.async_flag {
            // Use atomics in async mode for lock-free writes
            let head = self.head.load(Ordering::Relaxed);
            let next_head = (head + 1) % BUFFER_CAPACITY;

            // Check for buffer overflow
            if next_head != self.tail.load(Ordering::Acquire) {
                unsafe {
                    (*self.buffer[head].get()) = Some(formatted_msg); // Write log to buffer
                }
                self.head.store(next_head, Ordering::Release);
            } else {
                eprintln!("Buffer overflow, dropping log message");
            }

        } else if self.config.multi_threaded_flag {
            // Use mutex for thread-safe access when async is disabled but multi-threaded is enabled
            let _lock = self.mutex.as_ref().unwrap().lock().unwrap();
            self.write_log(&formatted_msg);  // Write to file and/or console

        } else {
            // Single-threaded, non-async mode: log immediately
            self.write_log(&formatted_msg);  // Write to file and/or console
        }
    }


    pub fn rotate_logs(&self, writer: &mut BufWriter<File>) {
        if let Some(ref path) = self.config.log_filepath {
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() > MAX_LOG_FILE_SIZE {
                    // Ensure no race condition by locking and checking the file size again
                    writer.flush().expect("Failed to flush before rotation");
    
                    // Prepare the rotated file path with a more precise timestamp to avoid conflicts
                    let rotated_path = format!("{}.{}", path, Local::now().format("%Y%m%d%H%M%S%.3f"));
    
                    // Rename the current log file
                    fs::rename(path, &rotated_path).expect("Failed to rotate log file");
    
                    // Re-open the log file and replace the BufWriter within the Mutex
                    let new_file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                        .expect("Failed to open new log file after rotation");
                    *writer = BufWriter::new(new_file);
    
                    // Write log rotation message
                    writeln!(
                        writer,
                        "\n\n--- Log rotated at {} ---",
                        Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
                    ).expect("Failed to write log rotation message");
                    writer.flush().expect("Failed to flush after log rotation");
                }
            }
        }
    }

    pub fn flush(&self) {
        if self.config.async_flag {
            // In async mode, ensure atomic and thread-safe flush
            let mut tail = self.tail.load(Ordering::Acquire); // Get the current tail position
            let head = self.head.load(Ordering::Acquire);     // Get the current head position
    
            while tail != head {
                // Load the message at the tail position atomically
                let log_msg_opt = unsafe { (*self.buffer[tail].get()).take() };
                
                if let Some(log_msg) = log_msg_opt {
                    self.write_log(&log_msg); // Write log to file and console
                }
    
                // Move the tail forward in a circular manner
                tail = (tail + 1) % BUFFER_CAPACITY;
    
                // Atomically store the updated tail position
                self.tail.store(tail, Ordering::Release);
            }
        } else if !self.config.multi_threaded_flag {
            // in multi-threaded mode, mutex ensures thread-safe logging, so no flush is required --
            // if neither async nor multi-threaded, just flush directly
            if let Some(ref writer) = self.file_writer {
                let mut writer_guard = writer.lock().unwrap();
                self.rotate_logs(&mut writer_guard);  // Check for rotation before flushing
                writer_guard.flush().expect("Failed to flush log file");
            }
        }
    }

    pub fn shutdown(&self) {
        if self.config.async_flag {
            self.should_run.store(false, Ordering::Relaxed);  // signal async thread to stop
        }
        
        self.flush();  // Ensure remaining logs are flushed before shutting down
        
        // write session start info if logging to file
        if let Some(ref writer) = self.file_writer {
            let mut writer_guard = writer.lock().unwrap();
            writeln!(
                writer_guard,
                "\n------ Session Ended at {} ------ \n",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            ).expect("Failed to write session start to log file");
            writer_guard.flush().expect("Failed to flush session start to log file");
        }
    }
}

// Global static pointer for logger
static LOGGER: AtomicPtr<Logger> = AtomicPtr::new(ptr::null_mut());

/* log_level, application_name, Opt<filepath>, Opt<console_flag>, Opt<async_flag>, Opt<multithreaded_flag> */
pub fn init(config: LogConfig) {
    let logger = Logger::new(config);

    let logger_ptr = Arc::into_raw(logger) as *mut Logger;

    LOGGER
        .compare_exchange(ptr::null_mut(), logger_ptr, Ordering::SeqCst, Ordering::SeqCst)
        .expect("Logger is already initialized!");

    init_panic_hook();
}


pub fn shutdown() {
    let logger_ptr = LOGGER.swap(ptr::null_mut(), Ordering::SeqCst);
    if !logger_ptr.is_null() {
        unsafe {
            let logger: Arc<Logger> = Arc::from_raw(logger_ptr); // Convert back to Arc<Logger>
            logger.shutdown();  // Flush and shutdown
        }
    }
}

pub fn with_logger<F: FnOnce(&Logger)>(f: F) {
    let logger_ptr = LOGGER.load(Ordering::SeqCst);
    if !logger_ptr.is_null() {
        unsafe { f(&*logger_ptr); }
    } else {
        panic!("Logger is not initialized!");
    }
}


#[cfg(not(feature = "performance"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        with_logger(|logger| logger.log(LogLevel::Info, &format!($($arg)*), CONSOLE_COLOR_BLUE));
    };
}

#[cfg(feature = "performance")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "performance"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        with_logger(|logger| logger.log(LogLevel::Warn, &format!($($arg)*), CONSOLE_COLOR_YELLOW));
    };
}

#[cfg(feature = "performance")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "performance"))]
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        with_logger(|logger| logger.log(LogLevel::Success, &format!($($arg)*), CONSOLE_BG_COLOR_GREEN));
    };
}

#[cfg(feature = "performance")]
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        with_logger(|logger| logger.log(LogLevel::Error, &format!($($arg)*), CONSOLE_COLOR_RED));
    };
}

#[macro_export]
macro_rules! crit {
    ($($arg:tt)*) => {
        with_logger(|logger| logger.log(LogLevel::Crit, &format!($($arg)*), CONSOLE_BG_COLOR_RED));
    };
}

#[macro_export]
macro_rules! log_flush {
    () => {
        with_logger(|logger| logger.flush());
    };
}

#[macro_export]
macro_rules! log_assert {
    ($cond:expr) => {
        if !$cond {
            crit!("Assertion failed: {}", stringify!($cond));
            panic!("Assertion failed: {}", stringify!($cond));
        }
    };
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            crit!("Assertion failed: {}", format!($($arg)*));
            panic!("Assertion failed: {}", format!($($arg)*));
        }
    };
}


pub fn init_panic_hook() {
    panic::set_hook(Box::new(|info| {
        // Extract panic location and message
        let location = info.location()
            .map(|loc| format!("file '{}' at line {}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = info.payload().downcast_ref::<&str>()
            .map(|msg| *msg)
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("Unknown panic message");

        // Log the panic information with high priority (e.g., Critical level)
        crit!("Panic occurred! Message: '{}' at {}", payload, location);

        // Flush the logger to ensure all logs are written before the program exits
        shutdown();
    }));
}
