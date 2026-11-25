use std::path;

use crate::arguments::Commands;
use crate::event::Event;
use crate::location::current_location;
use crate::location::Location;
use crate::madhab::Madhab;
use crate::method::{MethodVariant, ParamValue};
use crate::notification_urgency::NotifUrgency;
use crate::Arguments;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Utc};
use notify_rust::Urgency;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct PrayerConfig {
    method: MethodVariant,
    madhab: Madhab,
    fajr_mod: i8,
    dhuhr_mod: i8,
    asr_mod: i8,
    maghrib_mod: i8,
    isha_mod: i8,
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct NotificationConfig {
    notify_before: bool,
    urgency: NotifUrgency,
    icon: path::PathBuf,
    interval: u64,
}
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    location: Option<Location>,
    timezone: Option<String>,
    prayer: PrayerConfig,
    notification: NotificationConfig,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            notify_before: false,
            urgency: NotifUrgency::Critical,
            icon: default_icon(),
            interval: 20,
        }
    }
}

impl Config {
    // Generate a new Config from command line arguments
    pub fn new(args: &Arguments) -> Self {
        // println!("{:?}", args);

        let (program, config) = config_options();
        let config_res = confy::load::<Config>(program, config);
        if let Err(error) = &config_res {
            eprintln!("Error reading config file: {error}");
            if let Some(source) = error.source() {
                eprintln!("Caused by: {source}");
            }
        }
        Self::merge(args, config_res.unwrap_or_default())
    }

    // Layer command line arguments over a config read from file
    pub(crate) fn merge(args: &Arguments, config: Config) -> Self {
        let program = env!("CARGO_PKG_NAME");

        let mut interval = config.notification.interval;
        if let Some(Commands::Daemon(daemon)) = &args.command {
            if let Some(daemon_interval) = daemon.interval {
                interval = daemon_interval;
            }
        }
        if interval == 0 {
            interval = 1;
            eprintln!("Interval cannot be 0, setting it to 1 (the minimum value)");
        }

        let location: Location;
        if let (Some(latitude), Some(longitude)) = (args.latitude, args.longitude) {
            location = Location {
                lat: latitude,
                lon: longitude,
            };
        } else if let Some(cfg_location) = config.location {
            location = cfg_location;
        } else if !args.no_geolocation {
            if let Some(auto_location) = current_location() {
                location = auto_location;
            } else {
                eprintln!("No location provided in arguments or config file and impossible to get it automatically");
                eprintln!("Run the program using the latitude and longitude arguments or set them in the config file");
                eprintln!("Example : {program} --latitude <LAT> --longitude <LON>");
                std::process::exit(1);
            }
        } else {
            eprintln!("No location provided and --no-geolocation was set");
            eprintln!("Set it via --latitude/--longitude or in the config file");
            eprintln!("Example : {program} --latitude <LAT> --longitude <LON>");
            std::process::exit(1);
        }

        if let Err(err) = validate_location(&location) {
            eprintln!("Invalid location: {err}");
            std::process::exit(1);
        }

        let timezone = args.timezone.clone().or(config.timezone);
        if let Some(tz_str) = &timezone {
            if tz_str.parse::<chrono_tz::Tz>().is_err() {
                eprintln!("Invalid timezone '{tz_str}', falling back to the system timezone");
            }
        }

        Self {
            location: Some(location),
            timezone,
            prayer: PrayerConfig {
                method: args.method.unwrap_or(config.prayer.method),
                madhab: args.madhab.clone().unwrap_or(config.prayer.madhab),
                fajr_mod: args.fajr_mod.unwrap_or(config.prayer.fajr_mod),
                dhuhr_mod: args.dhuhr_mod.unwrap_or(config.prayer.dhuhr_mod),
                asr_mod: args.asr_mod.unwrap_or(config.prayer.asr_mod),
                maghrib_mod: args.maghrib_mod.unwrap_or(config.prayer.maghrib_mod),
                isha_mod: args.isha_mod.unwrap_or(config.prayer.isha_mod),
            },
            notification: NotificationConfig {
                notify_before: args
                    .notify_before
                    .unwrap_or(config.notification.notify_before),
                icon: args.icon.clone().unwrap_or(config.notification.icon),
                urgency: args.urgency.clone().unwrap_or(config.notification.urgency),
                interval,
            },
        }
    }

    pub fn lat(&self) -> f64 {
        if let Some(location) = &self.location {
            return location.lat;
        }
        0.
    }
    pub fn lon(&self) -> f64 {
        if let Some(location) = &self.location {
            return location.lon;
        }
        0.
    }

    fn tz(&self) -> Option<chrono_tz::Tz> {
        self.timezone.as_deref()?.parse().ok()
    }

    // UTC offset in hours, fractional for zones like Asia/Kolkata (+05:30)
    pub fn timezone_offset(&self, date: NaiveDate) -> f64 {
        match self.tz() {
            Some(tz) => utc_offset_hours(&tz, date),
            None => utc_offset_hours(&Local, date),
        }
    }

    // Wall-clock time in the configured timezone, the frame prayer times are computed in
    pub fn now(&self) -> NaiveDateTime {
        match self.tz() {
            Some(tz) => Utc::now().with_timezone(&tz).naive_local(),
            None => Local::now().naive_local(),
        }
    }

    pub fn fajr_param(&self) -> ParamValue {
        self.prayer.method.get().params.fajr
    }
    pub fn isha_param(&self) -> ParamValue {
        self.prayer.method.get().params.isha
    }
    pub fn shadow_multiplier(&self) -> u8 {
        self.prayer.madhab.shadow_multiplier()
    }

    pub fn offset(&self, event: Event) -> f64 {
        let minutes_mod = match event {
            Event::Fajr => self.prayer.fajr_mod,
            Event::Dhuhr => self.prayer.dhuhr_mod,
            Event::Asr => self.prayer.asr_mod,
            Event::Maghrib => self.prayer.maghrib_mod,
            Event::Isha => self.prayer.isha_mod,
            Event::Sunrise | Event::Sunset | Event::Midnight => 0,
        };
        f64::from(minutes_mod) / 60.
    }

    pub fn notify_before(&self) -> bool {
        self.notification.notify_before
    }
    pub fn urgency(&self) -> Urgency {
        self.notification.urgency.clone().into()
    }
    pub fn icon(&self) -> path::PathBuf {
        self.notification.icon.clone()
    }
    pub fn interval(&self) -> u64 {
        self.notification.interval
    }
}

pub(crate) fn validate_location(location: &Location) -> Result<(), String> {
    if !location.lat.is_finite() || !(-90.0..=90.0).contains(&location.lat) {
        return Err(format!(
            "latitude {} is out of range (expected -90..=90)",
            location.lat
        ));
    }
    if !location.lon.is_finite() || !(-180.0..=180.0).contains(&location.lon) {
        return Err(format!(
            "longitude {} is out of range (expected -180..=180)",
            location.lon
        ));
    }
    Ok(())
}

pub(crate) fn utc_offset_hours<Tz: TimeZone>(timezone: &Tz, date: NaiveDate) -> f64 {
    let noon = date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap_or(NaiveTime::MIN));
    let seconds = timezone
        .offset_from_local_datetime(&noon)
        .earliest()
        .map_or(0, |offset| offset.fix().local_minus_utc());
    f64::from(seconds) / 3600.
}

// Packages install the icon into the hicolor theme under this name
pub(crate) const ICON_NAME: &str = "prayer-times";

fn default_icon() -> path::PathBuf {
    if cfg!(debug_assertions) {
        path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/mosque-svgrepo-com.png")
    } else {
        path::PathBuf::from(ICON_NAME)
    }
}

pub fn config_options<'a>() -> (&'a str, &'a str) {
    const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
    (PROGRAM_NAME, "config")
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::event::Event;
    use crate::method::ParamValue;

    #[test]
    fn test_config_from_str() {
        let config_str = r#"
        timezone = "Europe/Paris"

        [location]
        lat = 48.8566
        lon = 2.3522

        [prayer]
        method = "KARACHI"
        madhab = "Hanafi"
        "#;

        let config: Config =
            toml::from_str(config_str).expect("Failed to parse config from string");

        assert_eq!(config.lat(), 48.8566);
        assert_eq!(config.lon(), 2.3522);
        assert_eq!(config.fajr_param(), ParamValue::Angle(18.0));
        assert_eq!(config.isha_param(), ParamValue::Angle(18.0));
        assert_eq!(config.shadow_multiplier(), 2);
        assert_eq!(config.offset(Event::Fajr), 0.0);
        assert_eq!(config.offset(Event::Dhuhr), 0.0);
        assert_eq!(config.offset(Event::Asr), 0.0);
        assert_eq!(config.offset(Event::Maghrib), 0.0);
        assert_eq!(config.offset(Event::Isha), 0.0);
        assert_eq!(config.timezone_offset(date(2025, 1, 15)), 1.);
        assert_eq!(config.icon(), super::default_icon());
    }

    #[test]
    fn test_default_icon_resolves() {
        let icon = super::default_icon();
        if cfg!(debug_assertions) {
            assert!(icon.is_file(), "{} should exist", icon.display());
        } else {
            assert_eq!(icon, std::path::PathBuf::from(super::ICON_NAME));
        }
    }

    #[test]
    fn test_validate_location_rejects_nan() {
        assert!(super::validate_location(&crate::location::Location {
            lat: f64::NAN,
            lon: 0.0,
        })
        .is_err());
        assert!(super::validate_location(&crate::location::Location {
            lat: 0.0,
            lon: f64::NAN,
        })
        .is_err());
    }

    #[test]
    fn test_validate_location_rejects_infinite() {
        assert!(super::validate_location(&crate::location::Location {
            lat: f64::INFINITY,
            lon: 0.0,
        })
        .is_err());
        assert!(super::validate_location(&crate::location::Location {
            lat: f64::NEG_INFINITY,
            lon: 0.0,
        })
        .is_err());
        assert!(super::validate_location(&crate::location::Location {
            lat: 0.0,
            lon: f64::INFINITY,
        })
        .is_err());
    }

    #[test]
    fn test_validate_location_rejects_out_of_range() {
        for (lat, lon) in [(91.0_f64, 0.0), (-91.0, 0.0), (0.0, 181.0), (0.0, -181.0)] {
            assert!(
                super::validate_location(&crate::location::Location { lat, lon }).is_err(),
                "expected reject for lat={lat}, lon={lon}"
            );
        }
    }

    #[test]
    fn test_validate_location_accepts_boundaries() {
        for (lat, lon) in [
            (90.0_f64, 0.0),
            (-90.0, 0.0),
            (0.0, 180.0),
            (0.0, -180.0),
            (0.0, 0.0),
        ] {
            assert!(
                super::validate_location(&crate::location::Location { lat, lon }).is_ok(),
                "expected accept for lat={lat}, lon={lon}"
            );
        }
    }

    fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_utc_offset_hours_fixed() {
        let d = date(2025, 1, 15);
        assert_eq!(super::utc_offset_hours(&chrono_tz::UTC, d), 0.);
        assert_eq!(super::utc_offset_hours(&chrono_tz::Asia::Riyadh, d), 3.);
        assert_eq!(super::utc_offset_hours(&chrono_tz::Asia::Kolkata, d), 5.5);
        assert_eq!(
            super::utc_offset_hours(&chrono_tz::Asia::Kathmandu, d),
            5.75
        );
        assert_eq!(
            super::utc_offset_hours(&chrono_tz::America::St_Johns, d),
            -3.5
        );
    }

    #[test]
    fn test_utc_offset_hours_dst() {
        let winter = date(2025, 1, 15);
        let summer = date(2025, 7, 15);
        assert_eq!(
            super::utc_offset_hours(&chrono_tz::Europe::Paris, winter),
            1.
        );
        assert_eq!(
            super::utc_offset_hours(&chrono_tz::Europe::Paris, summer),
            2.
        );
        assert_eq!(
            super::utc_offset_hours(&chrono_tz::America::New_York, winter),
            -5.
        );
        assert_eq!(
            super::utc_offset_hours(&chrono_tz::America::New_York, summer),
            -4.
        );
    }

    #[test]
    fn test_timezone_offset_invalid_falls_back_to_system() {
        let config: Config = toml::from_str(r#"timezone = "Not/A_Real_Tz""#).unwrap();
        let d = date(2025, 1, 15);
        assert_eq!(
            config.timezone_offset(d),
            super::utc_offset_hours(&chrono::Local, d)
        );
    }

    #[test]
    fn test_timezone_offset_valid() {
        let config: Config = toml::from_str(r#"timezone = "Asia/Kolkata""#).unwrap();
        assert_eq!(config.timezone_offset(date(2025, 1, 15)), 5.5);
    }

    #[test]
    fn test_offset_per_event_applies_minute_modifier() {
        use crate::arguments::Arguments;
        use crate::madhab::Madhab;
        use crate::method::MethodVariant;

        let args = Arguments {
            command: None,
            latitude: Some(0.0),
            longitude: Some(0.0),
            no_geolocation: true,
            timezone: Some("UTC".to_string()),
            method: Some(MethodVariant::MWL),
            madhab: Some(Madhab::Shafi),
            fajr_mod: Some(5),
            dhuhr_mod: Some(-10),
            asr_mod: Some(15),
            maghrib_mod: Some(-20),
            isha_mod: Some(25),
            notify_before: None,
            icon: None,
            urgency: None,
        };
        let config = Config::merge(&args, Config::default());

        assert!((config.offset(Event::Fajr) - 5.0 / 60.0).abs() < 1e-10);
        assert!((config.offset(Event::Dhuhr) + 10.0 / 60.0).abs() < 1e-10);
        assert!((config.offset(Event::Asr) - 15.0 / 60.0).abs() < 1e-10);
        assert!((config.offset(Event::Maghrib) + 20.0 / 60.0).abs() < 1e-10);
        assert!((config.offset(Event::Isha) - 25.0 / 60.0).abs() < 1e-10);

        // Sunrise / Sunset / Midnight are never offset, even when per-prayer
        // modifiers are set.
        assert_eq!(config.offset(Event::Sunrise), 0.0);
        assert_eq!(config.offset(Event::Sunset), 0.0);
        assert_eq!(config.offset(Event::Midnight), 0.0);
    }

    #[test]
    fn test_config_from_str_empty_uses_defaults() {
        let config: Config = toml::from_str("").expect("Failed to parse empty config");

        assert_eq!(config.fajr_param(), ParamValue::Angle(18.0));
        assert_eq!(config.isha_param(), ParamValue::Angle(17.0));
        assert_eq!(config.shadow_multiplier(), 1);
        assert!(!config.notify_before());
        assert_eq!(config.interval(), 20);
        for event in [
            Event::Fajr,
            Event::Dhuhr,
            Event::Asr,
            Event::Maghrib,
            Event::Isha,
        ] {
            assert_eq!(config.offset(event), 0.0);
        }
    }
}
