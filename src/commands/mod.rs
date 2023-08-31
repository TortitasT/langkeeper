mod key_generate;

use std::{
    env,
    io::{self},
    process::{exit, Command},
};

use crate::{
    db,
    logger::{log, LogLevel},
    mailer::send_verification_email,
    models, schema, start_server,
};

pub async fn parse_arguments() -> io::Result<()> {
    match env::args().nth(1) {
        Some(arg) => match arg.as_str() {
            "migrate" => {
                log("Running migrations...", LogLevel::Info);
                Command::new("diesel")
                    .args(&["migrations", "run"])
                    .output()
                    .expect("failed to execute process");
                log("Migrations completed", LogLevel::Info);

                exit(0)
            }
            "key:generate" => {
                key_generate::key_generate().await?;
            }
            "seed" => {
                let pool = db::get_connection_pool(None);

                log("Seeding database...", LogLevel::Info);
                db::seed_database(&pool);
                log("Database seeded", LogLevel::Info);

                exit(0)
            }
            "email:verify_current" => {
                use crate::schema::users::dsl::*;
                use diesel::prelude::*;

                let pool = db::get_connection_pool(None);
                let mut conn = pool.get().unwrap();

                let unverified_users = schema::users::dsl::users
                    .filter(verified.eq(0))
                    .load::<models::User>(&mut conn)
                    .unwrap();

                for user in unverified_users {
                    send_verification_email(&user);
                }
            }
            "serve" => {
                start_server().await?;
            }
            _ => {
                log("Invalid argument provided", LogLevel::Error);
                log("Usage: cargo run [migrate|seed|serve]", LogLevel::Error);

                exit(1)
            }
        },
        None => {
            log("No argument provided", LogLevel::Error);
            log("Usage: cargo run [migrate|seed|serve]", LogLevel::Error);

            exit(1)
        }
    };

    Ok(())
}
