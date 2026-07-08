//! Fraction-based target geometry.
//!
//! A [`Rect`] lives in a single top-left-origin, absolute coordinate space (points on
//! macOS, physical pixels on Windows — the per-OS shim converts into this space). Targets
//! are expressed as fractions `(x, y, w, h)` of a display's work area, each in `0.0..=1.0`,
//! so screen size, DPI, and origin never enter the math (idea.md §5.3).

use crate::core::actions::Action;

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

    /// Center point `(x, y)`.
    fn center(self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Whether the point `(px, py)` lies inside this rectangle (half-open on the far edges).
    fn contains(self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
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

/// Area of the overlap between two rectangles (0 if they don't overlap).
fn overlap_area(a: Rect, b: Rect) -> f64 {
    let x = ((a.x + a.w).min(b.x + b.w) - a.x.max(b.x)).max(0.0);
    let y = ((a.y + a.h).min(b.y + b.h) - a.y.max(b.y)).max(0.0);
    x * y
}

/// Pick the work area of the display a window is on (idea.md §5.3). The chosen display is the
/// one whose work area has the **largest overlap** with `window`. Ties — and windows that
/// overlap nothing (offscreen) — break toward the display that contains the window's center,
/// then toward the first display. With no displays it returns `window` unchanged (degenerate;
/// there is always at least one real display).
///
/// Screen size, DPI, and negative origins never enter this: it is pure rectangle math, so a
/// window on a secondary monitor at a negative offset selects that monitor correctly.
pub fn display_for(window: Rect, displays: &[Rect]) -> Rect {
    if displays.is_empty() {
        return window;
    }
    let (cx, cy) = window.center();
    // Rank each display by (overlap area, then whether it contains the window center). The
    // higher pair wins; a strict `>` keeps the first-listed display on a full tie.
    let score = |d: Rect| (overlap_area(window, d), d.contains(cx, cy));
    let mut best = 0;
    let mut best_score = score(displays[0]);
    for (i, &d) in displays.iter().enumerate().skip(1) {
        let s = score(d);
        if s > best_score {
            best = i;
            best_score = s;
        }
    }
    displays[best]
}

/// The geometry table: the absolute target rect for `action` at cycle `step`, for a window
/// currently at `_current` on the display with work area `work` (`_displays` carries all
/// display work areas for cross-display moves). `None` means the action's geometry isn't
/// implemented yet — the dispatcher logs that. Each action slice (007–019) fills in its arm.
pub fn target_for(
    action: Action,
    step: usize,
    _current: Rect,
    work: Rect,
    _displays: &[Rect],
) -> Option<Rect> {
    use Action::*;
    match action {
        // Halves cycle ½ → ⅔ → ⅓ (issue 005); `step` selects the size.
        LeftHalf | RightHalf | TopHalf | BottomHalf => {
            let cycle = action.as_half().unwrap().cycle();
            Some(fraction_to_rect(work, cycle[step % cycle.len()]))
        }
        // Center Half — centered half-width, full-height column (issue 007).
        CenterHalf => Some(fraction_to_rect(work, (0.25, 0.0, 0.5, 1.0))),
        // Thirds — direct full-height columns (issue 008).
        FirstThird => Some(fraction_to_rect(work, (0.0, 0.0, 1.0 / 3.0, 1.0))),
        CenterThird => Some(fraction_to_rect(work, (1.0 / 3.0, 0.0, 1.0 / 3.0, 1.0))),
        LastThird => Some(fraction_to_rect(work, (2.0 / 3.0, 0.0, 1.0 / 3.0, 1.0))),
        // Two-thirds — full-height columns, left/right anchored (issue 009).
        FirstTwoThirds => Some(fraction_to_rect(work, (0.0, 0.0, 2.0 / 3.0, 1.0))),
        LastTwoThirds => Some(fraction_to_rect(work, (1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0))),
        // Corners — quarter cells, half width × half height (issue 010).
        TopLeft => Some(fraction_to_rect(work, (0.0, 0.0, 0.5, 0.5))),
        TopRight => Some(fraction_to_rect(work, (0.5, 0.0, 0.5, 0.5))),
        BottomLeft => Some(fraction_to_rect(work, (0.0, 0.5, 0.5, 0.5))),
        BottomRight => Some(fraction_to_rect(work, (0.5, 0.5, 0.5, 0.5))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::actions::Action;

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

    // Two side-by-side 1000×800 displays: primary at origin, secondary to its right.
    const PRIMARY: Rect = Rect { x: 0.0, y: 0.0, w: 1000.0, h: 800.0 };
    const RIGHT: Rect = Rect { x: 1000.0, y: 0.0, w: 1000.0, h: 800.0 };

    #[test]
    fn display_for_window_fully_on_one() {
        let win = Rect::new(100.0, 100.0, 200.0, 200.0); // entirely inside PRIMARY
        assert_eq!(display_for(win, &[PRIMARY, RIGHT]), PRIMARY);
    }

    #[test]
    fn display_for_straddle_majority_wins() {
        // 800..1400: 200px in PRIMARY, 400px in RIGHT → RIGHT has the majority overlap.
        let win = Rect::new(800.0, 100.0, 600.0, 200.0);
        assert_eq!(display_for(win, &[PRIMARY, RIGHT]), RIGHT);
    }

    #[test]
    fn display_for_equal_overlap_breaks_to_center() {
        // 900..1100: 100px each side (equal overlap); center x=1000 is inside RIGHT
        // (half-open: 1000 is not in PRIMARY, is in RIGHT) → RIGHT wins the tie.
        let win = Rect::new(900.0, 100.0, 200.0, 200.0);
        assert_eq!(display_for(win, &[PRIMARY, RIGHT]), RIGHT);
    }

    #[test]
    fn display_for_offscreen_falls_back_to_first() {
        // Entirely left of both displays: no overlap, center in neither → first listed.
        let win = Rect::new(-500.0, 100.0, 200.0, 200.0);
        assert_eq!(display_for(win, &[PRIMARY, RIGHT]), PRIMARY);
    }

    #[test]
    fn display_for_selects_negative_origin_secondary() {
        // Secondary monitor to the LEFT at a negative origin (the dev setup); a window on it
        // must select it, not the primary.
        let secondary = Rect::new(-1440.0, 0.0, 1440.0, 900.0);
        let primary = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let win = Rect::new(-1000.0, 100.0, 300.0, 300.0);
        assert_eq!(display_for(win, &[primary, secondary]), secondary);
    }

    #[test]
    fn target_for_center_half() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        assert_eq!(
            target_for(Action::CenterHalf, 0, Rect::ZERO, work, &[]),
            Some(Rect::new(250.0, 0.0, 500.0, 800.0))
        );
    }

    #[test]
    fn target_for_left_half_cycles_with_step() {
        let work = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let at = |step| target_for(Action::LeftHalf, step, Rect::ZERO, work, &[]);
        assert_eq!(at(0), Some(Rect::new(0.0, 0.0, 600.0, 800.0))); // ½
        assert_eq!(at(1), Some(Rect::new(0.0, 0.0, 800.0, 800.0))); // ⅔
        assert_eq!(at(2), Some(Rect::new(0.0, 0.0, 400.0, 800.0))); // ⅓
    }

    #[test]
    fn target_for_unimplemented_action_is_none() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        // AlmostMaximize stays menu-only-unimplemented until issue 012.
        assert_eq!(target_for(Action::AlmostMaximize, 0, Rect::ZERO, work, &[]), None);
    }

    #[test]
    fn target_for_thirds_tile_full_width() {
        let work = Rect::new(0.0, 0.0, 900.0, 600.0);
        let f = |a| target_for(a, 0, Rect::ZERO, work, &[]).unwrap();
        assert_eq!(f(Action::FirstThird), Rect::new(0.0, 0.0, 300.0, 600.0));
        assert_eq!(f(Action::CenterThird), Rect::new(300.0, 0.0, 300.0, 600.0));
        assert_eq!(f(Action::LastThird), Rect::new(600.0, 0.0, 300.0, 600.0));
    }

    #[test]
    fn target_for_two_thirds() {
        let work = Rect::new(0.0, 0.0, 900.0, 600.0);
        let f = |a| target_for(a, 0, Rect::ZERO, work, &[]).unwrap();
        assert_eq!(f(Action::FirstTwoThirds), Rect::new(0.0, 0.0, 600.0, 600.0));
        assert_eq!(f(Action::LastTwoThirds), Rect::new(300.0, 0.0, 600.0, 600.0));
    }

    #[test]
    fn target_for_corners_tile_work_area() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let f = |a| target_for(a, 0, Rect::ZERO, work, &[]).unwrap();
        assert_eq!(f(Action::TopLeft), Rect::new(0.0, 0.0, 500.0, 400.0));
        assert_eq!(f(Action::TopRight), Rect::new(500.0, 0.0, 500.0, 400.0));
        assert_eq!(f(Action::BottomLeft), Rect::new(0.0, 400.0, 500.0, 400.0));
        assert_eq!(f(Action::BottomRight), Rect::new(500.0, 400.0, 500.0, 400.0));
    }
}
