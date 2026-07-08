//! Actions — the named window commands and the target geometry each maps to.
//!
//! For this slice this is just [`Half`] (moved out of the macOS shim). Issue 004 grows this
//! into the full `Action` registry; the pure rectangle math lives in [`super::geometry`].

/// Which half of the work area to snap the focused window to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    Left,
    Right,
    Top,
    Bottom,
}

impl Half {
    /// Target rectangle as a fraction `(x, y, w, h)` of the work area, each in `0.0..=1.0`.
    pub fn fraction(self) -> (f64, f64, f64, f64) {
        match self {
            Half::Left => (0.0, 0.0, 0.5, 1.0),
            Half::Right => (0.5, 0.0, 0.5, 1.0),
            Half::Top => (0.0, 0.0, 1.0, 0.5),
            Half::Bottom => (0.0, 0.5, 1.0, 0.5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_fractions_are_stable() {
        assert_eq!(Half::Left.fraction(), (0.0, 0.0, 0.5, 1.0));
        assert_eq!(Half::Right.fraction(), (0.5, 0.0, 0.5, 1.0));
        assert_eq!(Half::Top.fraction(), (0.0, 0.0, 1.0, 0.5));
        assert_eq!(Half::Bottom.fraction(), (0.0, 0.5, 1.0, 0.5));
    }
}
