use crate::{
    config::Config,
    notification::{notify_before_prayer, notify_prayer},
    prayer::Prayer,
    prayers,
};

pub fn run_daemon(config: &Config) {
    let mut daemon = PrayerDaemon::new(config);
    daemon.run();
}

struct PrayerDaemon<'a> {
    config: &'a Config,
    next_prayer: Prayer,
    is_notified_before: bool,
}

impl<'a> PrayerDaemon<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            config,
            next_prayer: prayers::next(config),
            is_notified_before: false,
        }
    }

    pub fn run(&mut self) {
        log::info!("Starting Prayer Times Background Process");
        loop {
            self.log_info();

            if self.next_prayer.time_has_passed() {
                self.handle_passed_prayer();
            } else if self.should_notify_before() {
                self.notify_before_prayer();
            }

            self.sleep();
        }
    }

    fn log_info(&self) {
        log::debug!("{}", self.next_prayer.text_duration());
        log::debug!("{}", self.next_prayer.text_time());
    }

    fn handle_passed_prayer(&mut self) {
        log::info!("Prayer time has passed");
        // Notification only if it's the current prayer.
        // If it's not the current prayer, it means that the system have been suspended
        // so we are currently in an other prayer
        if self.next_prayer == prayers::current(self.config) {
            notify_prayer(&self.next_prayer, self.config);
        }

        // Update next prayer
        self.next_prayer = prayers::next(self.config);
        self.is_notified_before = false;
    }

    fn should_notify_before(&self) -> bool {
        const MINUTES_BEFORE: i64 = 10;
        self.config.notify_before()
            && !self.is_notified_before
            && self.next_prayer.time_remaining() < chrono::Duration::minutes(MINUTES_BEFORE + 1)
    }

    fn notify_before_prayer(&mut self) {
        notify_before_prayer(
            &self.next_prayer,
            self.next_prayer.time_remaining(),
            self.config,
        );
        self.is_notified_before = true;
    }

    fn sleep(&self) {
        log::debug!("Next check in {} seconds", self.config.interval());
        std::thread::sleep(std::time::Duration::from_secs(self.config.interval()));
    }
}
