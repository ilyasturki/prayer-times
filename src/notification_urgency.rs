use notify_rust::Urgency;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::EnumString;

#[derive(Default, Debug, Clone, EnumString, Serialize, Deserialize)]
pub enum NotifUrgency {
    Low,
    Normal,
    #[default]
    Critical,
}
impl From<NotifUrgency> for Urgency {
    fn from(urgency: NotifUrgency) -> Self {
        match urgency {
            NotifUrgency::Low => Urgency::Low,
            NotifUrgency::Normal => Urgency::Normal,
            NotifUrgency::Critical => Urgency::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NotifUrgency;
    use notify_rust::Urgency;

    #[test]
    fn test_from_notif_urgency_maps_all_variants() {
        assert!(matches!(Urgency::from(NotifUrgency::Low), Urgency::Low));
        assert!(matches!(
            Urgency::from(NotifUrgency::Normal),
            Urgency::Normal
        ));
        assert!(matches!(
            Urgency::from(NotifUrgency::Critical),
            Urgency::Critical
        ));
    }

    #[test]
    fn test_default_is_critical() {
        assert!(matches!(NotifUrgency::default(), NotifUrgency::Critical));
    }
}
