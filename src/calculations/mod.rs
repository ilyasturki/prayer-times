use crate::{config::Config, event::Event, method::ParamValue};
use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime};

mod math;

fn positive_mod(value: f64, modulus: f64) -> f64 {
    let result = value - modulus * (value / modulus).floor();
    if result < 0. {
        result + modulus
    } else if result >= modulus {
        result - modulus
    } else {
        result
    }
}

fn normalize_degrees(angle: f64) -> f64 {
    positive_mod(angle, 360.)
}
fn normalize_hours(hour: f64) -> f64 {
    positive_mod(hour, 24.)
}

// https://orbital-mechanics.space/reference/julian-date.html
fn julian_day(date: NaiveDate) -> f64 {
    let day = date.day() as i32;
    let month = date.month() as i32;
    let year = date.year();

    let a = (month - 14) / 12;
    let b = 1461 * (year + 4800 + a);
    let c = 367 * (month - 2 - 12 * a);
    let e = (year + 4900 + a) / 100;

    (b / 4 + c / 12 - 3 * e / 4 + day - 32075) as f64
}

#[derive(Clone)]
pub struct AstronomicalMeasures {
    date: NaiveDate,
    fajr: f64,
    sunrise: f64,
    dhuhr: f64,
    asr: f64,
    sunset: f64,
    maghrib: f64,
    isha: f64,
    midnight: f64,
    // third_of_night: f64,
}
impl AstronomicalMeasures {
    pub fn new(date: NaiveDate, config: &Config) -> Self {
        // https://praytimes.org/calculation#astronomical_measures
        let (declination_of_sun, equation_of_time) = {
            let julian_day = julian_day(date);

            let d = julian_day - 2451545.0;

            let g = normalize_degrees(357.529 + 0.98560028 * d);
            let q = normalize_degrees(280.459 + 0.98564736 * d);
            let l = normalize_degrees(q + 1.915 * math::dsin(g) + 0.020 * math::dsin(2. * g));
            let e = 23.439 - 0.00000036 * d;
            let ra = math::darctan2(math::dcos(e) * math::dsin(l), math::dcos(l)) / 15.;

            let declination_of_sun = math::darcsin(math::dsin(e) * math::dsin(l));
            let equation_of_time = q / 15. - normalize_hours(ra);
            // println!("dec: {}, eq: {}", declination_of_sun, equation_of_time);

            (declination_of_sun, equation_of_time)
        };

        let solar_hour_angle = |angle: f64| -> f64 {
            let numerator =
                -math::dsin(angle) - math::dsin(config.lat()) * math::dsin(declination_of_sun);
            let denominator = math::dcos(config.lat()) * math::dcos(declination_of_sun);
            1. / 15. * math::darccos(numerator / denominator)
        };

        // https://praytimes.org/calculation#dhuhr
        let dhuhr = {
            let a = 12. + config.timezone_offset(date) as f64;
            let b = config.lon() / 15.;
            let c = equation_of_time;
            a - b - c
        };
        // https://praytimes.org/calculation#asr
        let asr = {
            let t = config.shadow_multiplier() as f64;
            let i = math::darccot(t + math::dtan((config.lat() - declination_of_sun).abs()));
            // let i = math::darccot(t + math::dtan(config.lat() - declination_of_sun));
            let a = math::dsin(i) - math::dsin(config.lat()) * math::dsin(declination_of_sun);
            let b = math::dcos(config.lat()) * math::dcos(declination_of_sun);
            1. / 15. * math::darccos(a / b)
        };

        // https://praytimes.org/calculation#sunrisesunset
        let sunrise = dhuhr - solar_hour_angle(0.833);
        let sunset = dhuhr + solar_hour_angle(0.833);
        let full_sunrise = if sunrise < sunset {
            sunrise + 24.
        } else {
            sunrise
        };
        let diff_night = full_sunrise - sunset;

        let fajr = {
            let fajr_param = config.fajr_param();
            match fajr_param {
                ParamValue::Angle(angle) => dhuhr - solar_hour_angle(angle),
                ParamValue::Minutes(minutes) => dhuhr - minutes as f64 / 60.,
            }
        };
        let isha = {
            let isha_param = config.isha_param();
            match isha_param {
                ParamValue::Angle(angle) => dhuhr + solar_hour_angle(angle),
                ParamValue::Minutes(minutes) => sunset + minutes as f64 / 60.,
            }
        };

        Self {
            date,
            sunrise,
            fajr: fajr + config.offset(Event::Fajr),
            dhuhr: dhuhr + config.offset(Event::Dhuhr),
            asr: dhuhr + asr + config.offset(Event::Asr),
            sunset,
            maghrib: sunset + config.offset(Event::Maghrib),
            isha: isha + config.offset(Event::Isha),
            midnight: sunset + 0.5 * diff_night,
            // third_of_night: sunset + 0.75 * diff_night,
        }
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    fn raw_time(&self, event: Event) -> f64 {
        match event {
            Event::Fajr => self.fajr,
            Event::Sunrise => self.sunrise,
            Event::Dhuhr => self.dhuhr,
            Event::Asr => self.asr,
            Event::Sunset => self.sunset,
            Event::Maghrib => self.maghrib,
            Event::Isha => self.isha,
            Event::Midnight => self.midnight,
        }
    }

    pub fn date_time(&self, event: Event) -> NaiveDateTime {
        let time = self.raw_time(event);

        // darccos/darcsin return NaN at extreme latitudes where the sun doesn't
        // reach the required altitude. Fall back to midnight so the program
        // keeps running; the caller's UI will still display the affected event.
        if !time.is_finite() {
            return NaiveDateTime::new(self.date(), NaiveTime::MIN);
        }

        let seconds = (time.rem_euclid(24.) * 3600.).clamp(0.0, 86_399.0) as u32;
        let naive_time =
            NaiveTime::from_num_seconds_from_midnight_opt(seconds, 0).unwrap_or(NaiveTime::MIN);

        let time_shift = (time / 24.).floor();
        let date = if time_shift >= 1. {
            self.date()
                .checked_add_days(Days::new(time_shift as u64))
                .unwrap_or_else(|| self.date())
        } else if time_shift < 0. {
            self.date()
                .checked_sub_days(Days::new(-time_shift as u64))
                .unwrap_or_else(|| self.date())
        } else {
            self.date()
        };
        NaiveDateTime::new(date, naive_time)
    }
}
