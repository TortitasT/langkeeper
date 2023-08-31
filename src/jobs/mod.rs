use std::{str::FromStr, time::Duration};

use chrono::Local;
use cron::Schedule;

use crate::mailer::send_mail;
use crate::schema::users::dsl::*;
use crate::{
    db,
    logger::{log, LogLevel},
};
use diesel::prelude::*;

pub fn init_jobs() {
    // init_weekly_report();
}

fn init_weekly_report() {
    actix_rt::spawn(async move {
        //                sec   min     hour    day of month    month   day of week   year
        // let expression = "0     0       9       *               *       1             *";
        let expression = "*/10     *       *       *               *       *             *"; // every 10 seconds for testing
        let schedule = Schedule::from_str(expression).unwrap();

        loop {
            let mut upcoming = schedule.upcoming(Local).take(1);

            actix_rt::time::sleep(Duration::from_millis(500)).await;

            let local = &Local::now();
            if let Some(datetime) = upcoming.next() {
                if datetime.timestamp() <= local.timestamp() {
                    log("Starting weekly report", LogLevel::Info);

                    let mut conn = db::get_connection_pool(None).get().unwrap();

                    let emails = users
                        .filter(crate::schema::users::verified.eq(1))
                        .filter(crate::schema::users::email.eq("victorgf2011@gmail.com"))
                        .select(crate::schema::users::email)
                        .load::<String>(&mut conn)
                        .unwrap();

                    for current_email in emails {
                        send_mail(
                            &current_email,
                            "Your weekly report",
                            &"This is a test, if you recieved this and are not the developer please send an email to victorgf2011@gmail.com and let me know :)".to_owned(),
                            false,
                        )
                        .await;
                    }
                }
            }
        }
    });
}
