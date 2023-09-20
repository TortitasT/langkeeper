use actix_jobs::{run_forever, Scheduler};

use self::weekly_report::WeeklyReportJob;

pub mod weekly_report;

pub fn init_jobs() {
    let mut scheduler = Scheduler::new();

    scheduler.add(Box::new(WeeklyReportJob {}));

    run_forever(scheduler);
}
