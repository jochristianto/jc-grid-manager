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

/// Default factor for Almost Maximize (idea.md §4): fill this fraction of the work area,
/// centered. Config hook — issue 020 makes it a user setting (`almost_maximize_factor`).
pub const ALMOST_MAXIMIZE_FACTOR: f64 = 0.9;

/// Smaller/Larger step: fraction of the work area added/removed per press. Config hook (020).
pub const RESIZE_STEP: f64 = 0.05;

/// Smaller floor: the window won't shrink below this fraction of the work area. Config hook (020).
pub const MIN_SIZE_FRACTION: f64 = 0.2;

/// Resize `win` by `dw`/`dh` (added to width/height) keeping its center fixed, then clamp into
/// `work`: size floored at `min_w`/`min_h` and capped at the work area, origin kept on-screen.
fn resize_around_center(win: Rect, work: Rect, dw: f64, dh: f64, min_w: f64, min_h: f64) -> Rect {
    let w = (win.w + dw).clamp(min_w.min(work.w), work.w);
    let h = (win.h + dh).clamp(min_h.min(work.h), work.h);
    let (cx, cy) = win.center();
    let x = (cx - w / 2.0).clamp(work.x, work.x + work.w - w);
    let y = (cy - h / 2.0).clamp(work.y, work.y + work.h - h);
    Rect::new(x, y, w, h)
}

/// The geometry table: the absolute target rect for `action` at cycle `step`, for a window
/// currently at `current` on the display with work area `work` (`_displays` carries all
/// display work areas for cross-display moves). `None` means the action's geometry isn't
/// implemented yet — the dispatcher logs that. Each action slice (007–019) fills in its arm.
pub fn target_for(
    action: Action,
    step: usize,
    current: Rect,
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
        // Maximize — fill the work area, not native fullscreen (issue 011).
        Maximize => Some(fraction_to_rect(work, (0.0, 0.0, 1.0, 1.0))),
        // Almost Maximize — centered, filling ALMOST_MAXIMIZE_FACTOR of the work area (012).
        AlmostMaximize => {
            let f = ALMOST_MAXIMIZE_FACTOR;
            Some(fraction_to_rect(work, ((1.0 - f) / 2.0, (1.0 - f) / 2.0, f, f)))
        }
        // Maximize Height — full work-area height, current width + x kept (issue 013). Setting
        // y/h to the work area's own values inherently clamps a vertically-offscreen window.
        MaximizeHeight => Some(Rect::new(current.x, work.y, current.w, work.h)),
        // Center — keep the current size, center in the work area; the max(…, 0) clamps a
        // window bigger than the work area to the top/left edge, not off-screen (issue 014).
        Center => {
            let x = work.x + ((work.w - current.w) / 2.0).max(0.0);
            let y = work.y + ((work.h - current.h) / 2.0).max(0.0);
            Some(Rect::new(x, y, current.w, current.h))
        }
        // Smaller / Larger — resize by RESIZE_STEP of the work area around the window's center,
        // capped at the work area and floored at MIN_SIZE_FRACTION of it (issue 015).
        Larger | Smaller => {
            let sign = if matches!(action, Larger) { 1.0 } else { -1.0 };
            let (dw, dh) = (sign * RESIZE_STEP * work.w, sign * RESIZE_STEP * work.h);
            let (min_w, min_h) = (MIN_SIZE_FRACTION * work.w, MIN_SIZE_FRACTION * work.h);
            Some(resize_around_center(current, work, dw, dh, min_w, min_h))
        }
        // Move to Edge — slide flush to an edge, no resize; other axis unchanged (issue 016).
        MoveLeft => Some(Rect::new(work.x, current.y, current.w, current.h)),
        MoveRight => Some(Rect::new(work.x + work.w - current.w, current.y, current.w, current.h)),
        MoveUp => Some(Rect::new(current.x, work.y, current.w, current.h)),
        MoveDown => Some(Rect::new(current.x, work.y + work.h - current.h, current.w, current.h)),
        // Fourths — full-height quarter-width columns (issue 017).
        FirstFourth => Some(fraction_to_rect(work, (0.0, 0.0, 0.25, 1.0))),
        SecondFourth => Some(fraction_to_rect(work, (0.25, 0.0, 0.25, 1.0))),
        ThirdFourth => Some(fraction_to_rect(work, (0.5, 0.0, 0.25, 1.0))),
        LastFourth => Some(fraction_to_rect(work, (0.75, 0.0, 0.25, 1.0))),
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
    fn target_for_almost_maximize_is_centered() {
        // Default factor 0.9 → 5% margins; general form ((1-f)/2,(1-f)/2,f,f). Compared with
        // tolerance because 0.9 is not exactly representable in f64.
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let got = target_for(Action::AlmostMaximize, 0, Rect::ZERO, work, &[]).unwrap();
        assert!((got.x - 50.0).abs() < 1e-6, "x={}", got.x);
        assert!((got.y - 40.0).abs() < 1e-6, "y={}", got.y);
        assert!((got.w - 900.0).abs() < 1e-6, "w={}", got.w);
        assert!((got.h - 720.0).abs() < 1e-6, "h={}", got.h);
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

    #[test]
    fn target_for_maximize_fills_work_area() {
        let work = Rect::new(100.0, 50.0, 1000.0, 800.0);
        assert_eq!(target_for(Action::Maximize, 0, Rect::ZERO, work, &[]), Some(work));
    }

    #[test]
    fn target_for_maximize_height_keeps_width_and_x() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let win = Rect::new(120.0, 300.0, 350.0, 200.0);
        // Full height, top of work area; width and x unchanged.
        assert_eq!(
            target_for(Action::MaximizeHeight, 0, win, work, &[]),
            Some(Rect::new(120.0, 0.0, 350.0, 800.0))
        );
    }

    #[test]
    fn target_for_center_keeps_size_and_centers() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let win = Rect::new(0.0, 0.0, 400.0, 200.0);
        assert_eq!(
            target_for(Action::Center, 0, win, work, &[]),
            Some(Rect::new(300.0, 300.0, 400.0, 200.0))
        );
    }

    #[test]
    fn target_for_center_clamps_oversized_window() {
        let work = Rect::new(10.0, 20.0, 500.0, 400.0);
        let win = Rect::new(0.0, 0.0, 800.0, 300.0); // wider than the work area
        // x clamps to work.x; y centers: 20 + (400-300)/2 = 70.
        assert_eq!(
            target_for(Action::Center, 0, win, work, &[]),
            Some(Rect::new(10.0, 70.0, 800.0, 300.0))
        );
    }

    #[test]
    fn target_for_larger_grows_around_center() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let win = Rect::new(300.0, 250.0, 400.0, 300.0); // center (500, 400)
        let got = target_for(Action::Larger, 0, win, work, &[]).unwrap();
        // +5%: 450×340, re-centered on (500,400) → x=275, y=230.
        assert!((got.w - 450.0).abs() < 1e-6, "w={}", got.w);
        assert!((got.h - 340.0).abs() < 1e-6, "h={}", got.h);
        assert!((got.x - 275.0).abs() < 1e-6, "x={}", got.x);
        assert!((got.y - 230.0).abs() < 1e-6, "y={}", got.y);
    }

    #[test]
    fn target_for_larger_caps_at_work_area() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let win = Rect::new(10.0, 10.0, 990.0, 790.0);
        let got = target_for(Action::Larger, 0, win, work, &[]).unwrap();
        assert!((got.w - 1000.0).abs() < 1e-6);
        assert!((got.h - 800.0).abs() < 1e-6);
        assert!(got.x >= -1e-6 && got.x + got.w <= 1000.0 + 1e-6);
    }

    #[test]
    fn target_for_smaller_floors_at_minimum() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0); // 20% floor → 200×160
        let win = Rect::new(400.0, 300.0, 210.0, 170.0);
        let got = target_for(Action::Smaller, 0, win, work, &[]).unwrap();
        assert!((got.w - 200.0).abs() < 1e-6, "w={}", got.w);
        assert!((got.h - 160.0).abs() < 1e-6, "h={}", got.h);
    }

    #[test]
    fn repeated_larger_converges_to_cap_without_runaway() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let mut win = Rect::new(400.0, 300.0, 200.0, 200.0);
        for _ in 0..40 {
            win = target_for(Action::Larger, 0, win, work, &[]).unwrap();
        }
        assert!((win.w - 1000.0).abs() < 1e-6);
        assert!((win.h - 800.0).abs() < 1e-6);
        assert!(win.x >= -1e-6 && win.x + win.w <= 1000.0 + 1e-6);
    }

    #[test]
    fn target_for_move_to_edge_keeps_size() {
        let work = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let win = Rect::new(300.0, 200.0, 250.0, 150.0);
        let f = |a| target_for(a, 0, win, work, &[]).unwrap();
        assert_eq!(f(Action::MoveLeft), Rect::new(0.0, 200.0, 250.0, 150.0));
        assert_eq!(f(Action::MoveRight), Rect::new(750.0, 200.0, 250.0, 150.0));
        assert_eq!(f(Action::MoveUp), Rect::new(300.0, 0.0, 250.0, 150.0));
        assert_eq!(f(Action::MoveDown), Rect::new(300.0, 650.0, 250.0, 150.0));
    }

    #[test]
    fn target_for_fourths_tile_width() {
        let work = Rect::new(0.0, 0.0, 800.0, 600.0);
        let f = |a| target_for(a, 0, Rect::ZERO, work, &[]).unwrap();
        assert_eq!(f(Action::FirstFourth), Rect::new(0.0, 0.0, 200.0, 600.0));
        assert_eq!(f(Action::SecondFourth), Rect::new(200.0, 0.0, 200.0, 600.0));
        assert_eq!(f(Action::ThirdFourth), Rect::new(400.0, 0.0, 200.0, 600.0));
        assert_eq!(f(Action::LastFourth), Rect::new(600.0, 0.0, 200.0, 600.0));
    }
}
