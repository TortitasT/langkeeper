use std::{
    env,
    fs::{copy, File},
};

use colored::Colorize;
use dotenvy::dotenv;

pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

pub fn log(message: &str, level: LogLevel) {
    dotenv().ok();

    let header = match level {
        LogLevel::Info => "[INFO]",
        LogLevel::Warn => "[WARN]",
        LogLevel::Error => "[ERROR]",
        LogLevel::Debug => "[DEBUG]",
    };

    println!(
        "{} {}",
        match level {
            LogLevel::Info => header.green(),
            LogLevel::Warn => header.yellow(),
            LogLevel::Error => header.red(),
            LogLevel::Debug => header.blue(),
        },
        message
    );

    match env::var("LOG_FILE") {
        Ok(log_file) => {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
                .unwrap();

            if File::metadata(&file).unwrap().len() > 1000000 {
                let rotated_file_name = format!("{}.1", log_file);
                copy(log_file, rotated_file_name).unwrap();

                file.set_len(0).unwrap();

                log(&format!("Rotated log file",), LogLevel::Info);
            }

            file.write_all(
                format!(
                    "[{}] {}: {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    header,
                    message
                )
                .as_bytes(),
            )
            .unwrap();
            file.write_all(b"\n").unwrap();
        }
        Err(_) => {}
    }
}
