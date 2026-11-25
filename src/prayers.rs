use crate::event::Event;
use crate::prayer::Prayer;
use crate::Config;
use chrono::{Days, NaiveDate, NaiveDateTime};

pub fn current(config: &Config) -> Prayer {
    next(config).previous()
}

pub fn next(config: &Config) -> Prayer {
    next_after(config, config.now())
}

// Yesterday's measures are included because their Midnight lands on today.
// Sunset is skipped: it shares Maghrib's time and current() never resolves to it.
fn next_after(config: &Config, now: NaiveDateTime) -> Prayer {
    let today = now.date();
    let yesterday = today.pred_opt().unwrap_or(today);
    let tomorrow = today.checked_add_days(Days::new(1)).unwrap_or(today);

    [yesterday, today, tomorrow]
        .into_iter()
        .flat_map(|date| Event::list().map(|event| Prayer::new(event, date, config)))
        .filter(|prayer| prayer.event() != Event::Sunset && prayer.date_time() >= now)
        .min_by_key(Prayer::date_time)
        .unwrap_or_else(|| Prayer::new(Event::Fajr, tomorrow, config))
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
    list_prayers_for_date(config, config.now().date())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{
        cairo_config, delhi_config, istanbul_config, makkah_config, medina_config, paris_config,
        paris_config_with_modifications,
    };

    fn paris_at(date: (i32, u32, u32), time: (u32, u32)) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(date.0, date.1, date.2)
            .unwrap()
            .and_hms_opt(time.0, time.1, 0)
            .unwrap()
    }

    // Paris 2025-10-02: Isha 20:35, Midnight 01:39 on 2025-10-03.
    #[test]
    fn test_next_after_isha_is_midnight_of_the_same_night() {
        let config = paris_config();
        let next = next_after(&config, paris_at((2025, 10, 2), (21, 0)));
        assert_eq!(next.event(), Event::Midnight);
        assert_eq!(
            next.date_time().date(),
            NaiveDate::from_ymd_opt(2025, 10, 3).unwrap()
        );
        assert_eq!(next.time().format("%H:%M").to_string(), "01:39");
    }

    #[test]
    fn test_next_before_midnight_after_calendar_day_change() {
        let config = paris_config();
        let next = next_after(&config, paris_at((2025, 10, 3), (0, 30)));
        assert_eq!(next.event(), Event::Midnight);
        assert_eq!(next.time().format("%H:%M").to_string(), "01:39");
    }

    #[test]
    fn test_next_after_midnight_is_fajr() {
        let config = paris_config();
        let next = next_after(&config, paris_at((2025, 10, 3), (2, 0)));
        assert_eq!(next.event(), Event::Fajr);
        assert_eq!(
            next.date_time().date(),
            NaiveDate::from_ymd_opt(2025, 10, 3).unwrap()
        );
    }

    // The daemon notifies a prayer once it passes and current() resolves to it.
    #[test]
    fn test_daemon_gate_matches_each_passed_prayer() {
        for (config, date) in [
            (
                paris_config(),
                NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(),
            ),
            (
                paris_config_with_modifications(),
                NaiveDate::from_ymd_opt(2024, 8, 10).unwrap(),
            ),
        ] {
            for event in [
                Event::Fajr,
                Event::Sunrise,
                Event::Dhuhr,
                Event::Asr,
                Event::Maghrib,
                Event::Isha,
            ] {
                let prayer = Prayer::new(event, date, &config);
                let before = prayer.date_time() - chrono::Duration::seconds(1);
                let after = prayer.date_time() + chrono::Duration::seconds(1);
                assert!(
                    next_after(&config, before) == prayer,
                    "{event} should be next"
                );
                assert!(
                    next_after(&config, after).previous() == prayer,
                    "{event} should be current once passed"
                );
            }
        }
    }

    // Paris 2025-10-02: Sunset and Maghrib both at 19:27.
    #[test]
    fn test_next_before_sunset_is_maghrib() {
        let config = paris_config();
        let next = next_after(&config, paris_at((2025, 10, 2), (19, 0)));
        assert_eq!(next.event(), Event::Maghrib);
        assert_eq!(next.previous().event(), Event::Asr);
    }

    #[test]
    fn test_next_during_the_day() {
        let config = paris_config();
        let next = next_after(&config, paris_at((2025, 10, 2), (14, 0)));
        assert_eq!(next.event(), Event::Asr);
        assert_eq!(next.time().format("%H:%M").to_string(), "16:48");
        assert_eq!(next.previous().event(), Event::Dhuhr);
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

    // Asia/Kolkata is UTC+05:30; aladhan rounds to the minute, so allow one minute.
    #[test]
    fn test_delhi_prayer_times_january_15_2025() {
        let config = delhi_config();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let prayers = list_prayers_for_date(&config, date);

        // curl -X GET "https://api.aladhan.com/v1/timings/15-01-2025?latitude=28.6139&longitude=77.2090&method=1&timezonestring=Asia/Kolkata" -H 'accept: application/json' | jq
        let expected = [
            "05:52", "07:15", "12:31", "15:26", "17:46", "17:46", "19:09", "00:31",
        ];
        for (prayer, expected) in prayers.iter().zip(expected) {
            let expected = chrono::NaiveTime::parse_from_str(expected, "%H:%M").unwrap();
            let delta = (prayer.time() - expected).num_seconds().abs();
            assert!(
                delta <= 60,
                "{} at {} expected {expected}",
                prayer.event(),
                prayer.time()
            );
        }
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
