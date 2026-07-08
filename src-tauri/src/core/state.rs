//! The snap state machine (idea.md §7) — pure logic, no window I/O.
//!
//! Repeating a directional shortcut cycles through sizes (½ → ⅔ → ⅓). The app remembers the
//! frame it last set (to tell "still where we put it") and a restore baseline (the frame
//! before the run began, for issue 006). Invalidation is automatic: if the window's current
//! frame no longer matches what we set, the next press is a fresh grab — no OS move/resize
//! listeners. Unit-testable with no real windows (§10).

use crate::core::actions::Action;
use crate::core::geometry::{fraction_to_rect, Rect};

/// Slack (points) for "the window is still where we put it". `set_frame` results aren't exact
/// (rounding; terminals and min-size windows clamp), so compare frames with a few px of give.
const MATCH_TOLERANCE: f64 = 5.0;

/// Whether two rectangles match within [`MATCH_TOLERANCE`] on every edge.
fn approx_eq(a: Rect, b: Rect) -> bool {
    (a.x - b.x).abs() <= MATCH_TOLERANCE
        && (a.y - b.y).abs() <= MATCH_TOLERANCE
        && (a.w - b.w).abs() <= MATCH_TOLERANCE
        && (a.h - b.h).abs() <= MATCH_TOLERANCE
}

/// One remembered snap run (§7). Single record for now; a small MRU list is a later upgrade.
#[derive(Debug, Clone, Copy)]
struct SnapRecord {
    action: Action,
    /// Index into the action's cycle table.
    step: usize,
    /// The frame we actually set, re-read after `set_frame`.
    last_set: Rect,
    /// The frame before this run's first snap — what Restore (006) returns to.
    baseline: Rect,
    /// Work area of the display this run is on; identifies the display for the same-display check.
    work_area: Rect,
}

/// Remembers the current snap run so repeats cycle and Restore can undo. Pure.
#[derive(Debug, Default)]
pub struct SnapState {
    record: Option<SnapRecord>,
}

impl SnapState {
    pub fn new() -> Self {
        SnapState { record: None }
    }

    /// Target frame for `action` (whose size `cycle` has ≥1 entry) applied to a window currently
    /// at `current_frame`, on the display with work area `work_area`.
    ///
    /// Continues the cycle only if the last run was the same action, on the same display, with
    /// the window still where we last put it; otherwise it's a fresh grab (step 0, new restore
    /// baseline). Call [`record_result`](Self::record_result) after `set_frame`.
    pub fn next_target(
        &mut self,
        action: Action,
        cycle: &[(f64, f64, f64, f64)],
        current_frame: Rect,
        work_area: Rect,
    ) -> Rect {
        debug_assert!(!cycle.is_empty(), "an action's cycle needs at least one step");

        let continues = self.record.is_some_and(|r| {
            r.action == action
                && approx_eq(r.work_area, work_area)
                && approx_eq(r.last_set, current_frame)
        });

        let (step, baseline) = match self.record {
            Some(r) if continues => ((r.step + 1) % cycle.len(), r.baseline),
            // Fresh grab: this pre-snap frame becomes the restore baseline.
            _ => (0, current_frame),
        };

        let target = fraction_to_rect(work_area, cycle[step]);
        self.record = Some(SnapRecord {
            action,
            step,
            last_set: target, // provisional until record_result stores the re-read frame
            baseline,
            work_area,
        });
        target
    }

    /// Store the frame the window actually landed at (re-read after `set_frame`), so the next
    /// press compares against reality — terminals / min-size windows don't land exactly.
    pub fn record_result(&mut self, actual_frame: Rect) {
        if let Some(r) = &mut self.record {
            r.last_set = actual_frame;
        }
    }

    /// The restore baseline for the current run, if any (what Restore returns the window to).
    pub fn baseline(&self) -> Option<Rect> {
        self.record.map(|r| r.baseline)
    }

    /// Forget the current run (Restore calls this after returning to the baseline).
    pub fn clear(&mut self) {
        self.record = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::actions::Half;

    const WORK: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1000.0,
        h: 800.0,
    };

    fn left() -> [(f64, f64, f64, f64); 3] {
        Half::Left.cycle()
    }

    #[test]
    fn fresh_grab_captures_baseline_at_step_zero() {
        let mut s = SnapState::new();
        let start = Rect::new(123.0, 45.0, 640.0, 480.0);
        let target = s.next_target(Action::LeftHalf, &left(), start, WORK);
        assert_eq!(target, fraction_to_rect(WORK, left()[0])); // ½
        assert_eq!(s.baseline(), Some(start));
    }

    #[test]
    fn repeat_same_action_cycles_and_keeps_baseline() {
        let mut s = SnapState::new();
        let start = Rect::new(10.0, 10.0, 300.0, 300.0);
        let t0 = s.next_target(Action::LeftHalf, &left(), start, WORK);
        s.record_result(t0);
        let t1 = s.next_target(Action::LeftHalf, &left(), t0, WORK);
        assert_eq!(t1, fraction_to_rect(WORK, left()[1])); // ⅔
        s.record_result(t1);
        let t2 = s.next_target(Action::LeftHalf, &left(), t1, WORK);
        assert_eq!(t2, fraction_to_rect(WORK, left()[2])); // ⅓
        s.record_result(t2);
        let t3 = s.next_target(Action::LeftHalf, &left(), t2, WORK);
        assert_eq!(t3, fraction_to_rect(WORK, left()[0])); // wraps to ½
        assert_eq!(s.baseline(), Some(start)); // baseline unchanged across the run
    }

    #[test]
    fn small_drift_within_tolerance_still_continues() {
        let mut s = SnapState::new();
        let t0 = s.next_target(Action::LeftHalf, &left(), Rect::new(0.0, 0.0, 300.0, 300.0), WORK);
        s.record_result(t0);
        // Next press: the window is a few px off what we set (rounding) but within tolerance.
        let slightly_off = Rect::new(t0.x + 3.0, t0.y - 2.0, t0.w + 1.0, t0.h - 1.0);
        let t1 = s.next_target(Action::LeftHalf, &left(), slightly_off, WORK);
        assert_eq!(t1, fraction_to_rect(WORK, left()[1])); // advanced, not reset
    }

    #[test]
    fn user_moved_window_resets_to_fresh() {
        let mut s = SnapState::new();
        let t0 = s.next_target(Action::LeftHalf, &left(), Rect::new(0.0, 0.0, 300.0, 300.0), WORK);
        s.record_result(t0);
        // Current frame no longer matches what we set (dragged well beyond tolerance) → fresh.
        let dragged = Rect::new(500.0, 400.0, 300.0, 300.0);
        let t1 = s.next_target(Action::LeftHalf, &left(), dragged, WORK);
        assert_eq!(t1, fraction_to_rect(WORK, left()[0])); // back to ½
        assert_eq!(s.baseline(), Some(dragged)); // new baseline captured
    }

    #[test]
    fn different_action_resets_to_fresh() {
        let mut s = SnapState::new();
        let t0 = s.next_target(Action::LeftHalf, &left(), Rect::new(0.0, 0.0, 300.0, 300.0), WORK);
        s.record_result(t0);
        let right = Half::Right.cycle();
        // Same frame, different action → fresh grab at the new action's step 0.
        let t1 = s.next_target(Action::RightHalf, &right, t0, WORK);
        assert_eq!(t1, fraction_to_rect(WORK, right[0]));
    }

    #[test]
    fn different_display_resets_to_fresh() {
        let mut s = SnapState::new();
        let t0 = s.next_target(Action::LeftHalf, &left(), Rect::new(0.0, 0.0, 300.0, 300.0), WORK);
        s.record_result(t0);
        // Same action + frame, but a different display (work area) → fresh grab.
        let other = Rect::new(-1440.0, 0.0, 1440.0, 900.0);
        let t1 = s.next_target(Action::LeftHalf, &left(), t0, other);
        assert_eq!(t1, fraction_to_rect(other, left()[0]));
    }

    #[test]
    fn restore_reads_baseline_then_clears() {
        let mut s = SnapState::new();
        let pre_snap = Rect::new(5.0, 5.0, 100.0, 100.0);
        s.next_target(Action::LeftHalf, &left(), pre_snap, WORK);
        // The Restore transition: read the pre-snap baseline, then clear the run.
        assert_eq!(s.baseline(), Some(pre_snap));
        s.clear();
        assert_eq!(s.baseline(), None);
    }
}
