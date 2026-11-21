//! Portable preference rules shared by platform frontends.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::System),
            1 => Some(Self::Light),
            2 => Some(Self::Dark),
            _ => None,
        }
    }

    pub fn is_dark(self, system_is_dark: bool) -> bool {
        match self {
            Self::System => system_is_dark,
            Self::Light => false,
            Self::Dark => true,
        }
    }
}

pub fn clamp_grid_scale(scale: f32) -> f32 {
    scale.clamp(0.5, 1.6)
}

#[cfg(test)]
mod tests {
    use super::{clamp_grid_scale, ThemeMode};

    #[test]
    fn theme_mode_respects_system_and_overrides() {
        assert!(ThemeMode::System.is_dark(true));
        assert!(!ThemeMode::System.is_dark(false));
        assert!(!ThemeMode::Light.is_dark(true));
        assert!(ThemeMode::Dark.is_dark(false));
        assert_eq!(ThemeMode::from_id(0), Some(ThemeMode::System));
        assert_eq!(ThemeMode::from_id(1), Some(ThemeMode::Light));
        assert_eq!(ThemeMode::from_id(2), Some(ThemeMode::Dark));
        assert_eq!(ThemeMode::from_id(3), None);
    }

    #[test]
    fn grid_scale_clamps_boundaries_and_preserves_nan() {
        assert_eq!(clamp_grid_scale(0.2), 0.5);
        assert_eq!(clamp_grid_scale(0.5), 0.5);
        assert_eq!(clamp_grid_scale(1.0), 1.0);
        assert_eq!(clamp_grid_scale(1.6), 1.6);
        assert_eq!(clamp_grid_scale(2.0), 1.6);
        assert!(clamp_grid_scale(f32::NAN).is_nan());
    }
}
