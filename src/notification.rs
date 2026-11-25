use crate::config::{Config, ICON_NAME};
use crate::prayer::Prayer;
use notify_rust::{Notification, Urgency};
use std::path::{Path, PathBuf};

fn send_notification(summary: String, urgency: Urgency, icon: PathBuf) {
    let mut notification = Notification::new(); // so the notification will live
    let notification = notification.summary(&summary).urgency(urgency);

    if let Some(icon_str) = resolve_icon(&icon).to_str() {
        notification.icon(icon_str);
    }

    match notification.show() {
        Ok(_) => log::info!("Notification sent"),
        Err(err) => eprintln!("Failed to send notification: {err}"),
    }
}

// A bare name is a theme icon; a missing path (such as the pre-0.4.5 default
// persisted in config files) falls back to the packaged theme icon.
fn resolve_icon(icon: &Path) -> PathBuf {
    if icon.components().count() <= 1 && !icon.is_absolute() {
        return icon.to_path_buf();
    }
    std::fs::canonicalize(icon).unwrap_or_else(|_| PathBuf::from(ICON_NAME))
}

pub fn notify_prayer(prayer: &Prayer, config: &Config) {
    let summary = format!("Adhan {}", prayer.event());
    send_notification(summary, config.urgency(), config.icon());
}

pub fn notify_before_prayer(prayer: &Prayer, duration: chrono::Duration, config: &Config) {
    let summary = format!(
        "Adhan {} in {} minutes",
        prayer.event(),
        duration.num_minutes()
    );
    send_notification(summary, Urgency::Low, config.icon());
}

#[cfg(test)]
mod tests {
    use super::{resolve_icon, ICON_NAME};
    use std::path::{Path, PathBuf};

    #[test]
    fn test_resolve_icon_keeps_theme_name() {
        assert_eq!(
            resolve_icon(Path::new("prayer-times")),
            PathBuf::from("prayer-times")
        );
    }

    #[test]
    fn test_resolve_icon_missing_path_falls_back_to_theme_icon() {
        let old_default = Path::new("/usr/share/icons/mosque-svgrepo-com.png");
        assert_eq!(resolve_icon(old_default), PathBuf::from(ICON_NAME));
    }

    #[test]
    fn test_resolve_icon_existing_path() {
        let asset = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/mosque-svgrepo-com.png");
        assert_eq!(resolve_icon(&asset), asset.canonicalize().unwrap());
    }
}
