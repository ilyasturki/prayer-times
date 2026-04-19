use crate::calculations::AstronomicalMeasures;
use crate::event::Event;
use crate::Config;
use chrono::{Days, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};

pub struct Prayer {
    event: Event,
    date: NaiveDateTime,
    measures: AstronomicalMeasures,
    config: Config,
}
impl PartialEq for Prayer {
    fn eq(&self, other: &Self) -> bool {
        self.event() == other.event() && self.date_time() == other.date_time()
    }
}
impl Prayer {
    fn new_from_measures(event: Event, measures: AstronomicalMeasures, config: &Config) -> Prayer {
        Self {
            event,
            date: measures.date_time(event),
            measures,
            config: config.clone(),
        }
    }
    pub fn new(event: Event, date: NaiveDate, config: &Config) -> Prayer {
        let measures = AstronomicalMeasures::new(date, config);
        Self::new_from_measures(event, measures, config)
    }

    fn new_from_date(&self, event: Event) -> Prayer {
        Self::new_from_measures(event, self.measures.clone(), &self.config)
    }

    pub fn event(&self) -> Event {
        self.event
    }
    pub fn date_time(&self) -> NaiveDateTime {
        self.date
    }
    pub fn date(&self) -> NaiveDate {
        self.date.date()
    }
    pub fn time(&self) -> NaiveTime {
        self.date.time()
    }

    pub fn previous(&self) -> Prayer {
        let previous_prayer = self.new_from_date(self.event.previous());
        if previous_prayer.time() <= self.time() {
            return previous_prayer;
        }

        let previous_date = self
            .date()
            .checked_sub_days(Days::new(1))
            .expect("Overflow when subtracting days");
        Self::new(self.event.previous(), previous_date, &self.config)
    }

    pub fn next(&self) -> Prayer {
        let next_prayer = self.new_from_date(self.event.next());
        if next_prayer.date_time().time() >= self.date_time().time() {
            return next_prayer;
        }

        let next_date = self
            .date()
            .checked_add_days(Days::new(1))
            .expect("Overflow when adding days");
        Self::new(self.event.next(), next_date, &self.config)
    }

    // Returns the time remaining for the next prayer to happen
    pub fn time_remaining(&self) -> Duration {
        let duration = self
            .date_time()
            .signed_duration_since(Local::now().naive_local());

        // The time remaining should not be negative
        if duration < Duration::zero() {
            return Duration::zero();
        }
        duration
    }

    // Returns true if the time of the prayer passed
    pub fn time_has_passed(&self) -> bool {
        self.time_remaining() <= Duration::zero()
    }

    pub fn text_duration(&self) -> String {
        let time_remaining = self.time_remaining();
        let in_or_since = if self.time_has_passed() {
            "since"
        } else {
            "in"
        };

        format!(
            "{} {in_or_since} {:02}H{:02}",
            self.event(),
            time_remaining.num_hours(),
            time_remaining.num_minutes() % 60
        )
    }
    pub fn text_time(&self) -> String {
        format!("{} at {}", self.event(), self.time())
        // format!("{} at {} the {}", self.event(), self.time(), self.date())
    }
}

#[cfg(test)]
mod tests {
    use super::Prayer;
    use crate::event::Event;
    use crate::test_util::paris_config;
    use chrono::NaiveDate;

    fn paris_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 10, 2).unwrap()
    }

    #[test]
    fn test_partial_eq_same_event_and_time() {
        let config = paris_config();
        let date = paris_date();
        let a = Prayer::new(Event::Fajr, date, &config);
        let b = Prayer::new(Event::Fajr, date, &config);
        assert!(a == b);

        let c = Prayer::new(Event::Dhuhr, date, &config);
        assert!(a != c);
    }

    #[test]
    fn test_previous_stays_same_day_when_possible() {
        let config = paris_config();
        let date = paris_date();
        let isha = Prayer::new(Event::Isha, date, &config);
        let previous = isha.previous();
        assert_eq!(previous.event(), Event::Maghrib);
        assert_eq!(previous.date(), date);
    }

    // Fajr's previous is Isha, which for Paris always belongs to the prior day.
    // Exercises the checked_sub_days branch at prayer.rs:54.
    #[test]
    fn test_previous_crosses_to_prior_day() {
        let config = paris_config();
        let date = paris_date();
        let fajr = Prayer::new(Event::Fajr, date, &config);
        let previous = fajr.previous();
        assert_eq!(previous.event(), Event::Isha);
        assert_eq!(
            previous.date(),
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()
        );
    }

    // Isha.next() is Midnight; Midnight's time-of-day (01:39 for Paris
    // 2025-10-02) is earlier than Isha's (20:35), so the next() helper takes
    // the checked_add_days branch at prayer.rs and rebuilds measures for the
    // following day.
    #[test]
    fn test_next_crosses_to_following_day() {
        let config = paris_config();
        let date = paris_date();
        let isha = Prayer::new(Event::Isha, date, &config);
        let next = isha.next();

        assert_eq!(next.event(), Event::Midnight);
        // Resulting midnight must be strictly after isha on the wall clock,
        // which is the invariant the cross-day logic exists to maintain.
        assert!(next.date_time() > isha.date_time());
    }

    #[test]
    fn test_text_time_format() {
        let config = paris_config();
        let fajr = Prayer::new(Event::Fajr, paris_date(), &config);
        let text = fajr.text_time();
        assert!(text.starts_with("Fajr at "));
        assert!(text.len() == "Fajr at HH:MM:SS".len());
    }

    // For a date far in the past, the duration sign is stable regardless of
    // wall-clock time, so these tests avoid flakiness without needing an
    // injected clock.
    #[test]
    fn test_text_duration_since_format_for_old_date() {
        let config = paris_config();
        let date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let fajr = Prayer::new(Event::Fajr, date, &config);
        let text = fajr.text_duration();
        assert!(text.starts_with("Fajr since "));
        assert!(text.contains('H'));
    }

    #[test]
    fn test_time_has_passed_for_old_date() {
        let config = paris_config();
        let date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let fajr = Prayer::new(Event::Fajr, date, &config);
        assert!(fajr.time_has_passed());
        assert_eq!(fajr.time_remaining(), chrono::Duration::zero());
    }
}
