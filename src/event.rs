use strum_macros::Display;

#[derive(Clone, Copy, Display, PartialEq)]
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
    pub fn next(&self) -> Event {
        use Event::{Asr, Dhuhr, Fajr, Isha, Maghrib, Midnight, Sunrise, Sunset};
        match self {
            Fajr => Sunrise,
            Sunrise => Dhuhr,
            Dhuhr => Asr,
            Asr => Maghrib,
            Sunset | Maghrib => Isha,
            Isha => Midnight,
            Midnight => Fajr,
        }
    }
}
