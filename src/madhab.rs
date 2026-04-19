use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;

#[derive(Default, Debug, Clone, EnumString, Serialize, Deserialize, EnumIter, Display)]
pub enum Madhab {
    #[default]
    Shafi,
    Hanafi,
}
impl Madhab {
    pub fn shadow_multiplier(&self) -> u8 {
        match self {
            Madhab::Shafi => 1,
            Madhab::Hanafi => 2,
        }
    }

    pub fn list_all() {
        for variant in Self::iter() {
            println!("{variant}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Madhab;

    #[test]
    fn test_shadow_multiplier() {
        assert_eq!(Madhab::Shafi.shadow_multiplier(), 1);
        assert_eq!(Madhab::Hanafi.shadow_multiplier(), 2);
    }

    #[test]
    fn test_default_is_shafi() {
        assert!(matches!(Madhab::default(), Madhab::Shafi));
    }
}
