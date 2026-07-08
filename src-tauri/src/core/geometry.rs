//! Fraction-based target geometry.
//!
//! A [`Rect`] lives in a single top-left-origin, absolute coordinate space (points on
//! macOS, physical pixels on Windows — the per-OS shim converts into this space). Targets
//! are expressed as fractions `(x, y, w, h)` of a display's work area, each in `0.0..=1.0`,
//! so screen size, DPI, and origin never enter the math (idea.md §5.3).

/// A rectangle in a top-left-origin, fraction-friendly absolute space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    /// The empty rectangle at the origin.
    #[allow(dead_code)] // used by the non-macOS stub today; by default-frame helpers soon.
    pub const ZERO: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    };

    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }
}

/// Turn a `(x, y, w, h)` fraction of `work` (each component in `0.0..=1.0`) into an
/// absolute [`Rect`] inside that work area. The work area's origin is carried through, so a
/// display at a negative offset (a monitor left of / above the primary) works unchanged.
pub fn fraction_to_rect(work: Rect, fraction: (f64, f64, f64, f64)) -> Rect {
    let (fx, fy, fw, fh) = fraction;
    Rect {
        x: work.x + fx * work.w,
        y: work.y + fy * work.h,
        w: fw * work.w,
        h: fh * work.h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_half_at_origin() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        assert_eq!(
            fraction_to_rect(work, (0.0, 0.0, 0.5, 1.0)),
            Rect::new(0.0, 0.0, 500.0, 800.0)
        );
    }

    #[test]
    fn bottom_half_at_origin() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        assert_eq!(
            fraction_to_rect(work, (0.0, 0.5, 1.0, 0.5)),
            Rect::new(0.0, 400.0, 1000.0, 400.0)
        );
    }

    #[test]
    fn right_half_on_display_at_negative_origin() {
        // Mirrors the dev multi-monitor setup: a secondary display left of / above the
        // primary sits at a negative origin. The offset must carry through the math.
        let work = Rect::new(-1440.0, -100.0, 1440.0, 900.0);
        assert_eq!(
            fraction_to_rect(work, (0.5, 0.0, 0.5, 1.0)),
            Rect::new(-720.0, -100.0, 720.0, 900.0)
        );
    }

    #[test]
    fn quarter_carries_offset() {
        let work = Rect::new(200.0, 300.0, 1600.0, 1000.0);
        assert_eq!(
            fraction_to_rect(work, (0.5, 0.5, 0.5, 0.5)),
            Rect::new(1000.0, 800.0, 800.0, 500.0)
        );
    }
}
