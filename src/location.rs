use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

// ip-api.com serves its free tier over plain HTTP only; with no IP in the path it locates the caller.
const GEOLOCATION_URL: &str = "http://ip-api.com/json/?fields=status,message,country,city,lat,lon";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    status: String,
    message: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    country: Option<String>,
}

fn fetch_geolocation() -> Result<String, ureq::Error> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();
    agent
        .get(GEOLOCATION_URL)
        .call()?
        .body_mut()
        .read_to_string()
}

fn parse_geolocation(body: &str) -> Result<(Location, IpApiResponse), String> {
    let response: IpApiResponse =
        serde_json::from_str(body).map_err(|err| format!("invalid response: {err}"))?;
    if response.status != "success" {
        return Err(response
            .message
            .unwrap_or_else(|| format!("status {}", response.status)));
    }
    let (Some(lat), Some(lon)) = (response.lat, response.lon) else {
        return Err("response has no coordinates".to_string());
    };
    Ok((Location { lat, lon }, response))
}

pub fn current_location() -> Option<Location> {
    let body = fetch_geolocation()
        .map_err(|err| log::warn!("Location auto-detection failed: {err}"))
        .ok()?;
    let (location, info) = parse_geolocation(&body)
        .map_err(|err| log::warn!("Location auto-detection failed: {err}"))
        .ok()?;

    log::info!(
        "Location auto-detected: lat={}, lon={}, city={}, country={}",
        location.lat,
        location.lon,
        info.city.unwrap_or_default(),
        info.country.unwrap_or_default(),
    );

    Some(location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_geolocation_success() {
        let body =
            r#"{"status":"success","country":"France","city":"Paris","lat":48.8566,"lon":2.3522}"#;
        let (location, info) = parse_geolocation(body).expect("should parse");
        assert_eq!(
            location,
            Location {
                lat: 48.8566,
                lon: 2.3522
            }
        );
        assert_eq!(info.city.as_deref(), Some("Paris"));
        assert_eq!(info.country.as_deref(), Some("France"));
    }

    #[test]
    fn test_parse_geolocation_fail_status() {
        let body = r#"{"status":"fail","message":"private range"}"#;
        assert_eq!(parse_geolocation(body).unwrap_err(), "private range");
    }

    #[test]
    fn test_parse_geolocation_missing_coordinates() {
        let body = r#"{"status":"success","country":"France"}"#;
        assert!(parse_geolocation(body).is_err());
    }

    #[test]
    fn test_parse_geolocation_invalid_json() {
        assert!(parse_geolocation("<html>").is_err());
    }
}
