use crate::event::Event;
use crate::prayer::Prayer;
use crate::Config;
use chrono::{Local, NaiveDate};

pub fn current(config: &Config) -> Prayer {
    next(config).previous()
}

pub fn next(config: &Config) -> Prayer {
    let current_date = Local::now().date_naive();
    let current_time = Local::now().time();

    for enum_prayer in Event::list() {
        let prayer = Prayer::new(enum_prayer, current_date, config);
        // Compare the prayer time with the current time
        if current_time <= prayer.date_time().time() {
            return prayer;
        }
    }

    // If no prayer in the current day found, return Time of fajr of tomorrow
    Prayer::new(Event::Isha, current_date, config).next()
}

pub fn list_prayers_for_date(config: &Config, date: NaiveDate) -> [Prayer; 8] {
    [
        Prayer::new(Event::Fajr, date, config),
        Prayer::new(Event::Sunrise, date, config),
        Prayer::new(Event::Dhuhr, date, config),
        Prayer::new(Event::Asr, date, config),
        Prayer::new(Event::Sunset, date, config),
        Prayer::new(Event::Maghrib, date, config),
        Prayer::new(Event::Isha, date, config),
        Prayer::new(Event::Midnight, date, config),
    ]
}

pub fn list_prayers(config: &Config) -> [Prayer; 8] {
    list_prayers_for_date(config, Local::now().date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::madhab::Madhab;
    use crate::method::MethodVariant;

    fn paris_config() -> Config {
        use crate::arguments::Arguments;

        let args = Arguments {
            command: None,
            latitude: Some(48.8566),
            longitude: Some(2.3522),
            timezone: Some("Europe/Paris".to_string()),
            method: Some(MethodVariant::FRANCE),
            madhab: Some(Madhab::Shafi),
            fajr_mod: None,
            dhuhr_mod: None,
            asr_mod: None,
            maghrib_mod: None,
            isha_mod: None,
            notify_before: None,
            icon: None,
            urgency: None,
        };

        Config::new(&args)
    }

    fn makkah_config() -> Config {
        use crate::arguments::Arguments;

        let args = Arguments {
            command: None,
            latitude: Some(21.42664),
            longitude: Some(39.82563),
            timezone: Some("Asia/Riyadh".to_string()),
            method: Some(MethodVariant::MAKKAH),
            madhab: Some(Madhab::Shafi),
            fajr_mod: None,
            dhuhr_mod: None,
            asr_mod: None,
            maghrib_mod: None,
            isha_mod: None,
            notify_before: None,
            icon: None,
            urgency: None,
        };

        Config::new(&args)
    }

    fn cairo_config() -> Config {
        use crate::arguments::Arguments;

        let args = Arguments {
            command: None,
            latitude: Some(30.0444),
            longitude: Some(31.2357),
            timezone: Some("Africa/Cairo".to_string()),
            method: Some(MethodVariant::EGYPT),
            madhab: Some(Madhab::Shafi),
            fajr_mod: None,
            dhuhr_mod: None,
            asr_mod: None,
            maghrib_mod: None,
            isha_mod: None,
            notify_before: None,
            icon: None,
            urgency: None,
        };

        Config::new(&args)
    }

    fn istanbul_config() -> Config {
        use crate::arguments::Arguments;

        let args = Arguments {
            command: None,
            latitude: Some(41.0082),
            longitude: Some(28.9784),
            timezone: Some("Europe/Istanbul".to_string()),
            method: Some(MethodVariant::TURKEY),
            madhab: Some(Madhab::Hanafi),
            fajr_mod: None,
            dhuhr_mod: None,
            asr_mod: None,
            maghrib_mod: None,
            isha_mod: None,
            notify_before: None,
            icon: None,
            urgency: None,
        };

        Config::new(&args)
    }

    fn medina_config() -> Config {
        use crate::arguments::Arguments;

        let args = Arguments {
            command: None,
            latitude: Some(24.5247),
            longitude: Some(39.5692),
            timezone: Some("Asia/Riyadh".to_string()),
            method: Some(MethodVariant::MAKKAH),
            madhab: Some(Madhab::Hanafi),
            fajr_mod: None,
            dhuhr_mod: None,
            asr_mod: None,
            maghrib_mod: None,
            isha_mod: None,
            notify_before: None,
            icon: None,
            urgency: None,
        };

        Config::new(&args)
    }

    fn paris_config_with_modifications() -> Config {
        use crate::arguments::Arguments;

        let args = Arguments {
            command: None,
            latitude: Some(48.8566),
            longitude: Some(2.3522),
            timezone: Some("Europe/Paris".to_string()),
            method: Some(MethodVariant::FRANCE),
            madhab: Some(Madhab::Shafi),
            fajr_mod: Some(5),
            dhuhr_mod: Some(-2),
            asr_mod: Some(3),
            maghrib_mod: Some(-1),
            isha_mod: Some(4),
            notify_before: None,
            icon: None,
            urgency: None,
        };

        Config::new(&args)
    }

    #[test]
    fn test_paris_prayer_times_october_2_2025() {
        let config = paris_config();
        let date = NaiveDate::from_ymd_opt(2025, 10, 2).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // curl -X GET "https://api.aladhan.com/v1/timings/02-10-2025?latitude=48.8566&longitude=2.3522&method=12&timezonestring=Europe%2FParis" -H 'accept: application/json' | jq
        assert_eq!(prayers[0].time().format("%H:%M").to_string(), "06:44");
        assert_eq!(prayers[1].time().format("%H:%M").to_string(), "07:52");
        assert_eq!(prayers[2].time().format("%H:%M").to_string(), "13:39");
        assert_eq!(prayers[3].time().format("%H:%M").to_string(), "16:48");
        assert_eq!(prayers[4].time().format("%H:%M").to_string(), "19:27");
        assert_eq!(prayers[5].time().format("%H:%M").to_string(), "19:27");
        assert_eq!(prayers[6].time().format("%H:%M").to_string(), "20:35");
        assert_eq!(prayers[7].time().format("%H:%M").to_string(), "01:39");
    }

    #[test]
    fn test_makkah_prayer_times_january_1_2022() {
        let config = makkah_config();
        let date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // curl -X GET "https://api.aladhan.com/v1/timings/01-01-2022?latitude=21.42664&longitude=39.82563&method=4&timezonestring=Asia%2FRiyadh" -H 'accept: application/json' | jq
        assert_eq!(prayers[0].time().format("%H:%M").to_string(), "05:37");
        assert_eq!(prayers[1].time().format("%H:%M").to_string(), "06:58");
        assert_eq!(prayers[2].time().format("%H:%M").to_string(), "12:24");
        assert_eq!(prayers[3].time().format("%H:%M").to_string(), "15:28");
        assert_eq!(prayers[4].time().format("%H:%M").to_string(), "17:49");
        assert_eq!(prayers[5].time().format("%H:%M").to_string(), "17:49");
        assert_eq!(prayers[6].time().format("%H:%M").to_string(), "19:19");
        assert_eq!(prayers[7].time().format("%H:%M").to_string(), "00:24");
    }

    #[test]
    fn test_cairo_prayer_times_march_15_2024() {
        let config = cairo_config();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // curl -X GET "https://api.aladhan.com/v1/timings/15-03-2024?latitude=30.0444&longitude=31.2357&method=5&timezonestring=Africa/Cairo" -H 'accept: application/json' | jq
        assert_eq!(prayers[0].time().format("%H:%M").to_string(), "04:37");
        assert_eq!(prayers[1].time().format("%H:%M").to_string(), "06:04");
        assert_eq!(prayers[2].time().format("%H:%M").to_string(), "12:03");
        assert_eq!(prayers[3].time().format("%H:%M").to_string(), "15:29");
        assert_eq!(prayers[4].time().format("%H:%M").to_string(), "18:03");
        assert_eq!(prayers[5].time().format("%H:%M").to_string(), "18:03");
        assert_eq!(prayers[6].time().format("%H:%M").to_string(), "19:20");
        assert_eq!(prayers[7].time().format("%H:%M").to_string(), "00:03");
    }

    #[test]
    fn test_istanbul_prayer_times_june_20_2024() {
        let config = istanbul_config();
        let date = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // curl -X GET "https://api.aladhan.com/v1/timings/20-06-2024?latitude=41.0082&longitude=28.9784&method=13&timezonestring=Europe/Istanbul&school=1" -H 'accept: application/json' | jq
        assert_eq!(prayers[0].time().format("%H:%M").to_string(), "03:24");
        assert_eq!(prayers[1].time().format("%H:%M").to_string(), "05:32");
        assert_eq!(prayers[2].time().format("%H:%M").to_string(), "13:05");
        assert_eq!(prayers[3].time().format("%H:%M").to_string(), "18:20");
        assert_eq!(prayers[4].time().format("%H:%M").to_string(), "20:39");
        assert_eq!(prayers[5].time().format("%H:%M").to_string(), "20:39");
        assert_eq!(prayers[6].time().format("%H:%M").to_string(), "22:38");
        assert_eq!(prayers[7].time().format("%H:%M").to_string(), "01:05");
    }

    #[test]
    fn test_medina_prayer_times_september_10_2023() {
        let config = medina_config();
        let date = NaiveDate::from_ymd_opt(2023, 9, 10).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // curl -X GET "https://api.aladhan.com/v1/timings/10-09-2023?latitude=24.5247&longitude=39.5692&method=4&timezonestring=Asia/Riyadh&school=1" -H 'accept: application/json' | jq
        assert_eq!(prayers[0].time().format("%H:%M").to_string(), "04:47");
        assert_eq!(prayers[1].time().format("%H:%M").to_string(), "06:06");
        assert_eq!(prayers[2].time().format("%H:%M").to_string(), "12:18");
        assert_eq!(prayers[3].time().format("%H:%M").to_string(), "16:46");
        assert_eq!(prayers[4].time().format("%H:%M").to_string(), "18:31");
        assert_eq!(prayers[5].time().format("%H:%M").to_string(), "18:31");
        assert_eq!(prayers[6].time().format("%H:%M").to_string(), "20:01");
        assert_eq!(prayers[7].time().format("%H:%M").to_string(), "00:18");
    }

    #[test]
    fn test_paris_prayer_times_with_modifications_august_10_2024() {
        let config = paris_config_with_modifications();
        let date = NaiveDate::from_ymd_opt(2024, 8, 10).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // Base times from API (without modifications):
        // curl -X GET "https://api.aladhan.com/v1/timings/10-08-2024?latitude=48.8566&longitude=2.3522&method=12&timezonestring=Europe/Paris" -H 'accept: application/json' | jq
        assert_eq!(prayers[0].time().format("%H:%M").to_string(), "05:21"); // Fajr: 05:16 + 5 min
        assert_eq!(prayers[1].time().format("%H:%M").to_string(), "06:37"); // Sunrise: no modification
        assert_eq!(prayers[2].time().format("%H:%M").to_string(), "13:53"); // Dhuhr: 13:55 - 2 min
        assert_eq!(prayers[3].time().format("%H:%M").to_string(), "17:59"); // Asr: 17:56 + 3 min
        assert_eq!(prayers[4].time().format("%H:%M").to_string(), "21:14"); // Sunset: no modification
        assert_eq!(prayers[5].time().format("%H:%M").to_string(), "21:13"); // Maghrib: 21:14 - 1 min
        assert_eq!(prayers[6].time().format("%H:%M").to_string(), "22:39"); // Isha: 22:35 + 4 min
        assert_eq!(prayers[7].time().format("%H:%M").to_string(), "01:55"); // Midnight: no modification
    }
}
