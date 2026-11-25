use strum_macros::Display;

#[derive(Clone, Copy, Debug, Display, PartialEq)]
pub enum Event {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
    Sunset,
    Midnight,
}
impl Event {
    pub fn list() -> [Event; 8] {
        use Event::{Asr, Dhuhr, Fajr, Isha, Maghrib, Midnight, Sunrise, Sunset};
        [Fajr, Sunrise, Dhuhr, Asr, Sunset, Maghrib, Isha, Midnight]
    }
    pub fn previous(&self) -> Self {
        use Event::{Asr, Dhuhr, Fajr, Isha, Maghrib, Midnight, Sunrise, Sunset};
        match self {
            Sunrise => Fajr,
            Dhuhr => Sunrise,
            Asr => Dhuhr,
            Sunset | Maghrib => Asr,
            Isha => Maghrib,
            Fajr | Midnight => Isha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn test_previous_cycles_through_all_events() {
        assert_eq!(Event::Fajr.previous(), Event::Isha);
        assert_eq!(Event::Sunrise.previous(), Event::Fajr);
        assert_eq!(Event::Dhuhr.previous(), Event::Sunrise);
        assert_eq!(Event::Asr.previous(), Event::Dhuhr);
        assert_eq!(Event::Maghrib.previous(), Event::Asr);
        assert_eq!(Event::Isha.previous(), Event::Maghrib);
        assert_eq!(Event::Midnight.previous(), Event::Isha);
    }

    #[test]
    fn test_sunset_previous_is_asr() {
        assert_eq!(Event::Sunset.previous(), Event::Asr);
    }

    #[test]
    fn test_list_returns_all_eight_in_display_order() {
        use Event::{Asr, Dhuhr, Fajr, Isha, Maghrib, Midnight, Sunrise, Sunset};
        assert_eq!(
            Event::list(),
            [Fajr, Sunrise, Dhuhr, Asr, Sunset, Maghrib, Isha, Midnight]
        );
    }
}
