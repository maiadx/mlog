
use mlog::*;
use std::sync::Arc;
use std::fs;
use std::thread;
use std::time::Duration;
use chrono::Local;




#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused)]
    fn get_test_log_path() -> String {
        format!("test_log_{}.log", Local::now().timestamp())
    }
    #[test]
    #[allow(unreachable_code)]

    fn test_default() {

        let log_config = LogConfig {
            time_format : "%H:%M:%S".to_string(),
            ..Default::default()
        };

        // Initialize the logger
        mlog::init(log_config);


        info!("This is an info message.");
        warn!("This is a warning.");
        error!("Error Code : ({})", 2);
        crit!("This is a critical error :/");
        success!("Operation successful :)");

        // panic!("aaaaaaaaa"); // tests implemented panic hook

        shutdown();
        }
    
    //     #[test]
    // fn test_single_threaded_non_async() {
    //     let config = LogConfig {
    //         log_level: LogLevel::Info,
    //         application_name: "single_thread_non_async_test".to_string(),
    //         log_filepath: Some(get_test_log_path()),  // Write logs to a file
    //         console_flag: true,  // Enable console logging
    //         async_flag: false,   // Disable async logging
    //         multi_threaded_flag: false,  // Disable multi-threading
    //         time_format: "%Y-%m-%d %H:%M:%S".to_string(),
    //     };

    //     let logger = Logger::new(config);

    //     // Perform some logging
    //     logger.log(LogLevel::Info, "This is an info message", CONSOLE_COLOR_BLUE);
    //     logger.log(LogLevel::Warn, "This is a warning", CONSOLE_COLOR_YELLOW);
    //     logger.flush(); // Flush to make sure logs are written

    //     // Assert that the log file exists and contains the logged messages
    //     if let Some(ref log_filepath) = logger.config.log_filepath {
    //         let log_contents = fs::read_to_string(log_filepath).expect("Failed to read log file");
    //         assert!(log_contents.contains("This is an info message"));
    //         assert!(log_contents.contains("This is a warning"));
    //     }
    // }

    // // Test case for single-threaded async logger with console output enabled
    // #[test]
    // fn test_single_threaded_async() {
    //     let config = LogConfig {
    //         log_level: LogLevel::Info,
    //         application_name: "single_thread_async_test".to_string(),
    //         log_filepath: Some(get_test_log_path()),
    //         console_flag: true,
    //         async_flag: true,  // Enable async logging
    //         multi_threaded_flag: false,  // Single-threaded mode
    //         time_format: "%Y-%m-%d %H:%M:%S".to_string(),
    //     };

    //     let logger = Logger::new(config);

    //     // Log messages asynchronously
    //     logger.log(LogLevel::Info, "This is an async info message", CONSOLE_COLOR_BLUE);
    //     logger.log(LogLevel::Warn, "This is an async warning", CONSOLE_COLOR_YELLOW);

    //     // Give the async thread some time to write the log
    //     thread::sleep(Duration::from_secs(1));
    //     logger.flush();  // Ensure that everything is written

    //     // Check if the logs are written to the file
    //     if let Some(ref log_filepath) = logger.config.log_filepath {
    //         let log_contents = fs::read_to_string(log_filepath).expect("Failed to read log file");
    //         assert!(log_contents.contains("This is an async info message"));
    //         assert!(log_contents.contains("This is an async warning"));
    //     }
    // }

    // // Test case for multi-threaded non-async logger with console output enabled
    // #[test]
    // fn test_multi_threaded_non_async() {
    //     let config = LogConfig {
    //         log_level: LogLevel::Info,
    //         application_name: "multi_thread_non_async_test".to_string(),
    //         log_filepath: Some(get_test_log_path()),
    //         console_flag: true,
    //         async_flag: false,  // Non-async mode
    //         multi_threaded_flag: true,  // Enable multi-threading
    //         time_format: "%Y-%m-%d %H:%M:%S".to_string(),
    //     };

    //     let logger = Logger::new(config);

    //     // Perform logging from multiple threads
    //     let logger_clone = Arc::clone(&logger);
    //     let handle = thread::spawn(move || {
    //         logger_clone.log(LogLevel::Info, "Log from thread", CONSOLE_COLOR_BLUE);
    //     });

    //     logger.log(LogLevel::Warn, "Log from main thread", CONSOLE_COLOR_YELLOW);
    //     handle.join().unwrap();

    //     // Flush and ensure logs are written
    //     logger.flush();

    //     // Verify the log file contents
    //     if let Some(ref log_filepath) = logger.config.log_filepath {
    //         let log_contents = fs::read_to_string(log_filepath).expect("Failed to read log file");
    //         assert!(log_contents.contains("Log from thread"));
    //         assert!(log_contents.contains("Log from main thread"));
    //     }
    // }

    // Test case for multi-threaded async logger with console output enabled
    // #[test]
    // fn test_multi_threaded_async() {
    //     let config = LogConfig {
    //         log_level: LogLevel::Info,
    //         application_name: "multi_thread_async_test".to_string(),
    //         log_filepath: Some(get_test_log_path()),
    //         console_flag: true,
    //         async_flag: true,  // Enable async logging
    //         multi_threaded_flag: true,  // Enable multi-threading
    //         time_format: "%Y-%m-%d %H:%M:%S".to_string(),
    //     };

    //     let logger = Logger::new(config);

    //     // Perform logging from multiple threads asynchronously
    //     let logger_clone = Arc::clone(&logger);
    //     let handle = thread::spawn(move || {
    //         logger_clone.log(LogLevel::Info, "Async log from thread", CONSOLE_COLOR_BLUE);
    //     });

    //     logger.log(LogLevel::Warn, "Async log from main thread", CONSOLE_COLOR_YELLOW);
    //     handle.join().unwrap();

    //     // Give the async thread time to flush logs
    //     thread::sleep(Duration::from_secs(1));
    //     logger.flush();

    //     // Verify the log file contents
    //     if let Some(ref log_filepath) = logger.config.log_filepath {
    //         let log_contents = fs::read_to_string(log_filepath).expect("Failed to read log file");
    //         assert!(log_contents.contains("Async log from thread"));
    //         assert!(log_contents.contains("Async log from main thread"));
    //     }
    // }

    // // Test case for single-threaded with mutex locking
    // #[test]
    // fn test_single_threaded_with_mutex() {
    //     let config = LogConfig {
    //         log_level: LogLevel::Info,
    //         application_name: "single_thread_mutex_test".to_string(),
    //         log_filepath: Some(get_test_log_path()),
    //         console_flag: true,
    //         async_flag: false,
    //         multi_threaded_flag: true,  // Multi-threaded mode with a mutex for safety
    //         time_format: "%Y-%m-%d %H:%M:%S".to_string(),
    //     };

    //     let logger = Logger::new(config);

    //     // Log messages and ensure mutex handles the access safely
    //     logger.log(LogLevel::Info, "Mutex log info message", CONSOLE_COLOR_BLUE);
    //     logger.log(LogLevel::Warn, "Mutex log warning", CONSOLE_COLOR_YELLOW);

    //     logger.flush();  // Ensure everything is written

    //     // Verify the log file contents
    //     if let Some(ref log_filepath) = logger.config.log_filepath {
    //         let log_contents = fs::read_to_string(log_filepath).expect("Failed to read log file");
    //         assert!(log_contents.contains("Mutex log info message"));
    //         assert!(log_contents.contains("Mutex log warning"));
    //     }
    // }

//     #[test]
// fn test_log_file_rotation() {
//     let log_file_path = get_test_log_path();  // Helper function to get the log file path

//     let config = LogConfig {
//         log_level: LogLevel::Info,
//         application_name: "file_rotation_test".to_string(),
//         log_filepath: Some(log_file_path.clone()),  // Log file path
//         console_flag: false,   // Disable console logging
//         async_flag: false,     // Disable async logging for simplicity
//         multi_threaded_flag: false,  // Single-threaded mode
//         time_format: "%Y-%m-%d %H:%M:%S".to_string(),  // Time format
//     };

//     // Set a smaller maximum log file size for testing (e.g., 1KB)
//     const TEST_MAX_LOG_FILE_SIZE: u64 = 1024; // 1KB
//     let logger = Logger::new(config);

//     // Write multiple large log entries to exceed the file size limit
//     for i in 0..100 {
//         logger.log(LogLevel::Info, &format!("This is log entry number {}", i), CONSOLE_COLOR_BLUE);
//     }

//     logger.flush(); // Flush all logs to the file

//     // Verify that the original log file has been rotated and a new file is created
//     let metadata = fs::metadata(&log_file_path).expect("Failed to get metadata for log file");
//     assert!(metadata.len() < TEST_MAX_LOG_FILE_SIZE, "Log file size should be smaller than the max file size after rotation");

//     // Check that the rotated log file exists with the expected naming format
//     let rotated_log_file = format!("{}.{}", log_file_path, Local::now().format("%Y%m%d%H%M%S"));
//     assert!(fs::metadata(&rotated_log_file).is_ok(), "Rotated log file should exist");

//     // Verify that the old log file contains logs from before the rotation
//     let rotated_log_contents = fs::read_to_string(rotated_log_file).expect("Failed to read rotated log file");
//     assert!(rotated_log_contents.contains("This is log entry number 0"), "Rotated log file should contain earlier log entries");

//     // Verify that the new log file contains only the most recent entries
//     let new_log_contents = fs::read_to_string(&log_file_path).expect("Failed to read new log file");
//     assert!(new_log_contents.contains(&format!("This is log entry number {}", 99)), "New log file should contain the most recent log entries");

//     // Clean up (optional)
//     fs::remove_file(log_file_path).expect("Failed to clean up log file");
// }
}
