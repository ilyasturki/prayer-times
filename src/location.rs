use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

fn public_ip() -> Option<String> {
    let mut ip = None;
    // List all of the machine's network interfaces
    for iface in get_if_addrs::get_if_addrs().ok()? {
        if iface.is_loopback() {
            continue;
        }
        let ip_addr = iface.ip().to_string();
        if ip_addr.starts_with("192.168") {
            continue;
        }
        ip = Some(ip_addr);
    }

    ip
}

pub fn current_location() -> Option<Location> {
    let info = geolocation::find(public_ip()?.as_str()).ok()?;
    let location = Location {
        lat: info.latitude.parse().ok()?,
        lon: info.longitude.parse().ok()?,
    };

    log::info!(
        "Location auto-detected: lat={}, lon={}, city={}, country={}",
        location.lat,
        location.lon,
        info.city,
        info.country,
    );

    Some(location)
}
