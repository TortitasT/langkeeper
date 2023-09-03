use self::weekly_report::init_weekly_report;

pub mod weekly_report;

pub fn init_jobs() {
    init_weekly_report();
}
