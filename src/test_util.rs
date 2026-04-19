#![cfg(test)]

use crate::arguments::Arguments;
use crate::config::Config;
use crate::madhab::Madhab;
use crate::method::MethodVariant;

pub struct Fixture {
    pub lat: f64,
    pub lon: f64,
    pub tz: &'static str,
    pub method: MethodVariant,
    pub madhab: Madhab,
    pub fajr_mod: Option<i8>,
    pub dhuhr_mod: Option<i8>,
    pub asr_mod: Option<i8>,
    pub maghrib_mod: Option<i8>,
    pub isha_mod: Option<i8>,
}

pub fn build(f: Fixture) -> Config {
    let args = Arguments {
        command: None,
        latitude: Some(f.lat),
        longitude: Some(f.lon),
        no_geolocation: true,
        timezone: Some(f.tz.to_string()),
        method: Some(f.method),
        madhab: Some(f.madhab),
        fajr_mod: f.fajr_mod,
        dhuhr_mod: f.dhuhr_mod,
        asr_mod: f.asr_mod,
        maghrib_mod: f.maghrib_mod,
        isha_mod: f.isha_mod,
        notify_before: None,
        icon: None,
        urgency: None,
    };
    Config::new(&args)
}

pub fn simple(
    lat: f64,
    lon: f64,
    tz: &'static str,
    method: MethodVariant,
    madhab: Madhab,
) -> Config {
    build(Fixture {
        lat,
        lon,
        tz,
        method,
        madhab,
        fajr_mod: None,
        dhuhr_mod: None,
        asr_mod: None,
        maghrib_mod: None,
        isha_mod: None,
    })
}

pub fn paris_config() -> Config {
    simple(
        48.8566,
        2.3522,
        "Europe/Paris",
        MethodVariant::FRANCE,
        Madhab::Shafi,
    )
}

pub fn makkah_config() -> Config {
    simple(
        21.42664,
        39.82563,
        "Asia/Riyadh",
        MethodVariant::MAKKAH,
        Madhab::Shafi,
    )
}

pub fn cairo_config() -> Config {
    simple(
        30.0444,
        31.2357,
        "Africa/Cairo",
        MethodVariant::EGYPT,
        Madhab::Shafi,
    )
}

pub fn istanbul_config() -> Config {
    simple(
        41.0082,
        28.9784,
        "Europe/Istanbul",
        MethodVariant::TURKEY,
        Madhab::Hanafi,
    )
}

pub fn medina_config() -> Config {
    simple(
        24.5247,
        39.5692,
        "Asia/Riyadh",
        MethodVariant::MAKKAH,
        Madhab::Hanafi,
    )
}

pub fn paris_config_with_modifications() -> Config {
    build(Fixture {
        lat: 48.8566,
        lon: 2.3522,
        tz: "Europe/Paris",
        method: MethodVariant::FRANCE,
        madhab: Madhab::Shafi,
        fajr_mod: Some(5),
        dhuhr_mod: Some(-2),
        asr_mod: Some(3),
        maghrib_mod: Some(-1),
        isha_mod: Some(4),
    })
}
