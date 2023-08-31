use std::{
    fs::File,
    io::{self, Read, Write},
    process::exit,
};

use rand::Rng;

use crate::logger::{log, LogLevel};

pub async fn key_generate() -> io::Result<()> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*%#@!~";
    const PASSWORD_LEN: usize = 30;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let mut file = File::open(".env")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut new_contents = String::new();
    for line in contents.lines() {
        if line.starts_with("JWT_SECRET=") {
            new_contents.push_str(&format!("JWT_SECRET={}\n", password));
        } else {
            new_contents.push_str(&format!("{}\n", line));
        }
    }

    let mut file = File::create(".env")?;
    file.write_all(new_contents.as_bytes())?;
    file.sync_all()?;

    log("JWT secret generated", LogLevel::Info);
    exit(0)
}
