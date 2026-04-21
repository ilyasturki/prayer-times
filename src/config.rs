use std::path;

use crate::arguments::Commands;
use crate::event::Event;
use crate::location::current_location;
use crate::location::Location;
use crate::madhab::Madhab;
use crate::method::{MethodVariant, ParamValue};
use crate::notification_urgency::NotifUrgency;
use crate::Arguments;
use chrono::Local;
use chrono::{NaiveDate, NaiveTime, TimeZone};
use chrono_tz::OffsetComponents;
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
        let config: Config = config_res.unwrap_or_default();

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

        Self {
            location: Some(location),
            timezone: args.timezone.clone().or(config.timezone),
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

    pub fn timezone_offset(&self, date: NaiveDate) -> i64 {
        match &self.timezone {
            Some(tz_str) => parse_timezone_string(tz_str, date),
            None => system_timezone_offset(),
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

pub(crate) fn parse_timezone_string(tz_str: &str, date: NaiveDate) -> i64 {
    if let Ok(tz) = tz_str.parse::<chrono_tz::Tz>() {
        return timezone_to_offset(tz, date);
    }

    let offset = system_timezone_offset();
    eprintln!("Invalid timezone '{tz_str}', falling back to system timezone GMT{offset:+}");
    offset
}

pub(crate) fn timezone_to_offset(timezone: chrono_tz::Tz, date: NaiveDate) -> i64 {
    let noon = date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    let local_time = timezone.from_local_datetime(&noon).unwrap();
    let offset = local_time.offset();
    let total_offset = offset.base_utc_offset() + offset.dst_offset();
    total_offset.num_hours()
}

pub(crate) fn system_timezone_offset() -> i64 {
    let local_time = Local::now();
    let offset = local_time.offset().local_minus_utc();
    let offset_hours = offset / 3600;
    i64::from(offset_hours)
}

// Get the icon of the notification that should be sent
fn default_icon() -> path::PathBuf {
    let assets_path = if cfg!(debug_assertions) {
        path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
    } else {
        path::PathBuf::from("/usr/share/icons")
    };

    assets_path.join("mosque-svgrepo-com.png")
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
        [location]
        lat = 48.8566
        lon = 2.3522
        timezone = "Europe/Paris"

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
        assert!(config.icon().ends_with("assets/mosque-svgrepo-com.png"));
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

    #[test]
    fn test_timezone_to_offset_fixed() {
        let d = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert_eq!(super::timezone_to_offset(chrono_tz::UTC, d), 0);
        assert_eq!(super::timezone_to_offset(chrono_tz::Asia::Riyadh, d), 3);
        // Asia/Kolkata is +05:30; the hour-resolution implementation truncates
        // the half-hour — document that.
        assert_eq!(super::timezone_to_offset(chrono_tz::Asia::Kolkata, d), 5);
    }

    #[test]
    fn test_timezone_to_offset_dst() {
        let winter = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let summer = chrono::NaiveDate::from_ymd_opt(2025, 7, 15).unwrap();

        assert_eq!(
            super::timezone_to_offset(chrono_tz::Europe::Paris, winter),
            1
        );
        assert_eq!(
            super::timezone_to_offset(chrono_tz::Europe::Paris, summer),
            2
        );

        assert_eq!(
            super::timezone_to_offset(chrono_tz::America::New_York, winter),
            -5
        );
        assert_eq!(
            super::timezone_to_offset(chrono_tz::America::New_York, summer),
            -4
        );
    }

    #[test]
    fn test_parse_timezone_string_invalid_falls_back_to_system() {
        let d = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let fallback = super::parse_timezone_string("Not/A_Real_Tz", d);
        assert_eq!(fallback, super::system_timezone_offset());
    }

    #[test]
    fn test_parse_timezone_string_valid() {
        let d = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert_eq!(super::parse_timezone_string("UTC", d), 0);
        assert_eq!(super::parse_timezone_string("Asia/Riyadh", d), 3);
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
        let config = Config::new(&args);

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
