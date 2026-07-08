//! Actions — the named window commands.
//!
//! [`Action`] enumerates every command in idea.md §4. It carries only identity here: a
//! stable string `id` (config keys, issue 020), a menu `label` (tray menu, issue 024), and
//! — for the four directional halves — a mapping to [`Half`]. The target geometry for each
//! action is filled in by its own slice (005–019); the default key bindings live in
//! [`crate::shortcuts`] (they depend on the global-shortcut plugin, so they stay out of this
//! pure module). Pure and unit-testable with no real windows (§10).

/// Which half of the work area to snap the focused window to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    Left,
    Right,
    Top,
    Bottom,
}

impl Half {
    /// The size cycle for repeating this directional half (idea.md §4/§7): ½ → ⅔ → ⅓ — widths
    /// for Left/Right, heights for Top/Bottom. Each entry is an `(x, y, w, h)` fraction of the
    /// work area; step 0 (½) is the first press.
    pub fn cycle(self) -> [(f64, f64, f64, f64); 3] {
        match self {
            Half::Left => [
                (0.0, 0.0, 1.0 / 2.0, 1.0),
                (0.0, 0.0, 2.0 / 3.0, 1.0),
                (0.0, 0.0, 1.0 / 3.0, 1.0),
            ],
            Half::Right => [
                (1.0 / 2.0, 0.0, 1.0 / 2.0, 1.0),
                (1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0),
                (2.0 / 3.0, 0.0, 1.0 / 3.0, 1.0),
            ],
            Half::Top => [
                (0.0, 0.0, 1.0, 1.0 / 2.0),
                (0.0, 0.0, 1.0, 2.0 / 3.0),
                (0.0, 0.0, 1.0, 1.0 / 3.0),
            ],
            Half::Bottom => [
                (0.0, 1.0 / 2.0, 1.0, 1.0 / 2.0),
                (0.0, 1.0 / 3.0, 1.0, 2.0 / 3.0),
                (0.0, 2.0 / 3.0, 1.0, 1.0 / 3.0),
            ],
        }
    }
}

/// Every window command in idea.md §4. Cycling sizes (½ → ⅔ → ⅓) are a state-machine
/// concern (issue 005), not separate actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Halves
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    CenterHalf,
    // Corners (quarters)
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    // Thirds
    FirstThird,
    CenterThird,
    LastThird,
    FirstTwoThirds,
    LastTwoThirds,
    // Sizing
    Maximize,
    AlmostMaximize,
    MaximizeHeight,
    Smaller,
    Larger,
    Center,
    Restore,
    // Displays
    NextDisplay,
    PreviousDisplay,
    // Move to edge (no resize)
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    // Fourths
    FirstFourth,
    SecondFourth,
    ThirdFourth,
    LastFourth,
    // Sixths (3 across × 2 down)
    SixthTopLeft,
    SixthTopCenter,
    SixthTopRight,
    SixthBottomLeft,
    SixthBottomCenter,
    SixthBottomRight,
}

impl Action {
    /// Every action, in menu order. Source of truth for iteration (registration, tray menu).
    pub const ALL: [Action; 37] = [
        Action::LeftHalf,
        Action::RightHalf,
        Action::TopHalf,
        Action::BottomHalf,
        Action::CenterHalf,
        Action::TopLeft,
        Action::TopRight,
        Action::BottomLeft,
        Action::BottomRight,
        Action::FirstThird,
        Action::CenterThird,
        Action::LastThird,
        Action::FirstTwoThirds,
        Action::LastTwoThirds,
        Action::Maximize,
        Action::AlmostMaximize,
        Action::MaximizeHeight,
        Action::Smaller,
        Action::Larger,
        Action::Center,
        Action::Restore,
        Action::NextDisplay,
        Action::PreviousDisplay,
        Action::MoveLeft,
        Action::MoveRight,
        Action::MoveUp,
        Action::MoveDown,
        Action::FirstFourth,
        Action::SecondFourth,
        Action::ThirdFourth,
        Action::LastFourth,
        Action::SixthTopLeft,
        Action::SixthTopCenter,
        Action::SixthTopRight,
        Action::SixthBottomLeft,
        Action::SixthBottomCenter,
        Action::SixthBottomRight,
    ];

    /// Stable, kebab-case identifier used as a config key. Never change these once shipped.
    #[allow(dead_code)] // consumed by issue 020 (config keys) / 029 (shortcut editor)
    pub fn id(self) -> &'static str {
        match self {
            Action::LeftHalf => "left-half",
            Action::RightHalf => "right-half",
            Action::TopHalf => "top-half",
            Action::BottomHalf => "bottom-half",
            Action::CenterHalf => "center-half",
            Action::TopLeft => "top-left",
            Action::TopRight => "top-right",
            Action::BottomLeft => "bottom-left",
            Action::BottomRight => "bottom-right",
            Action::FirstThird => "first-third",
            Action::CenterThird => "center-third",
            Action::LastThird => "last-third",
            Action::FirstTwoThirds => "first-two-thirds",
            Action::LastTwoThirds => "last-two-thirds",
            Action::Maximize => "maximize",
            Action::AlmostMaximize => "almost-maximize",
            Action::MaximizeHeight => "maximize-height",
            Action::Smaller => "smaller",
            Action::Larger => "larger",
            Action::Center => "center",
            Action::Restore => "restore",
            Action::NextDisplay => "next-display",
            Action::PreviousDisplay => "previous-display",
            Action::MoveLeft => "move-left",
            Action::MoveRight => "move-right",
            Action::MoveUp => "move-up",
            Action::MoveDown => "move-down",
            Action::FirstFourth => "first-fourth",
            Action::SecondFourth => "second-fourth",
            Action::ThirdFourth => "third-fourth",
            Action::LastFourth => "last-fourth",
            Action::SixthTopLeft => "sixth-top-left",
            Action::SixthTopCenter => "sixth-top-center",
            Action::SixthTopRight => "sixth-top-right",
            Action::SixthBottomLeft => "sixth-bottom-left",
            Action::SixthBottomCenter => "sixth-bottom-center",
            Action::SixthBottomRight => "sixth-bottom-right",
        }
    }

    /// Human-readable label for menus (idea.md §4 naming).
    pub fn label(self) -> &'static str {
        match self {
            Action::LeftHalf => "Left Half",
            Action::RightHalf => "Right Half",
            Action::TopHalf => "Top Half",
            Action::BottomHalf => "Bottom Half",
            Action::CenterHalf => "Center Half",
            Action::TopLeft => "Top Left",
            Action::TopRight => "Top Right",
            Action::BottomLeft => "Bottom Left",
            Action::BottomRight => "Bottom Right",
            Action::FirstThird => "First Third",
            Action::CenterThird => "Center Third",
            Action::LastThird => "Last Third",
            Action::FirstTwoThirds => "First Two Thirds",
            Action::LastTwoThirds => "Last Two Thirds",
            Action::Maximize => "Maximize",
            Action::AlmostMaximize => "Almost Maximize",
            Action::MaximizeHeight => "Maximize Height",
            Action::Smaller => "Smaller",
            Action::Larger => "Larger",
            Action::Center => "Center",
            Action::Restore => "Restore",
            Action::NextDisplay => "Next Display",
            Action::PreviousDisplay => "Previous Display",
            Action::MoveLeft => "Move Left",
            Action::MoveRight => "Move Right",
            Action::MoveUp => "Move Up",
            Action::MoveDown => "Move Down",
            Action::FirstFourth => "First Fourth",
            Action::SecondFourth => "Second Fourth",
            Action::ThirdFourth => "Third Fourth",
            Action::LastFourth => "Last Fourth",
            Action::SixthTopLeft => "Top-Left Sixth",
            Action::SixthTopCenter => "Top-Center Sixth",
            Action::SixthTopRight => "Top-Right Sixth",
            Action::SixthBottomLeft => "Bottom-Left Sixth",
            Action::SixthBottomCenter => "Bottom-Center Sixth",
            Action::SixthBottomRight => "Bottom-Right Sixth",
        }
    }

    /// The four directional halves map to a [`Half`]; every other action returns `None`.
    /// Used by the dispatcher to route the already-implemented halves through geometry.
    pub fn as_half(self) -> Option<Half> {
        match self {
            Action::LeftHalf => Some(Half::Left),
            Action::RightHalf => Some(Half::Right),
            Action::TopHalf => Some(Half::Top),
            Action::BottomHalf => Some(Half::Bottom),
            _ => None,
        }
    }

    /// Reverse of [`id`](Action::id): look an action up by its stable id (config loading).
    #[allow(dead_code)] // consumed by issue 020 (config loading)
    pub fn from_id(id: &str) -> Option<Action> {
        Action::ALL.into_iter().find(|a| a.id() == id)
    }

    /// Number of size-cycle steps: the four directional halves cycle ½→⅔→⅓ (3); every other
    /// action is a single, direct placement (1). Drives the snap state machine (§7).
    pub fn cycle_len(self) -> usize {
        if self.as_half().is_some() {
            3
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_cycle_starts_at_one_half() {
        assert_eq!(Half::Left.cycle()[0], (0.0, 0.0, 0.5, 1.0));
        assert_eq!(Half::Right.cycle()[0], (0.5, 0.0, 0.5, 1.0));
        assert_eq!(Half::Top.cycle()[0], (0.0, 0.0, 1.0, 0.5));
        assert_eq!(Half::Bottom.cycle()[0], (0.0, 0.5, 1.0, 0.5));
    }

    #[test]
    fn half_cycle_advances_half_two_thirds_one_third() {
        // Left: left-anchored widths ½ → ⅔ → ⅓.
        let left = Half::Left.cycle();
        assert_eq!(left[0].2, 0.5);
        assert!((left[1].2 - 2.0 / 3.0).abs() < 1e-9);
        assert!((left[2].2 - 1.0 / 3.0).abs() < 1e-9);
        // Right's ⅓ step is right-anchored at x = 2/3; Bottom's ⅓ step is at y = 2/3.
        assert!((Half::Right.cycle()[2].0 - 2.0 / 3.0).abs() < 1e-9);
        assert!((Half::Bottom.cycle()[2].1 - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn every_action_has_unique_nonempty_id_and_label() {
        let mut ids = std::collections::HashSet::new();
        for action in Action::ALL {
            let id = action.id();
            assert!(!id.is_empty(), "{action:?} has an empty id");
            assert!(!action.label().is_empty(), "{action:?} has an empty label");
            assert!(ids.insert(id), "duplicate id {id:?}");
        }
        assert_eq!(ids.len(), Action::ALL.len());
    }

    #[test]
    fn id_round_trips_through_from_id() {
        for action in Action::ALL {
            assert_eq!(Action::from_id(action.id()), Some(action));
        }
        assert_eq!(Action::from_id("not-a-real-action"), None);
    }

    #[test]
    fn only_directional_halves_map_to_a_half() {
        assert_eq!(Action::LeftHalf.as_half(), Some(Half::Left));
        assert_eq!(Action::RightHalf.as_half(), Some(Half::Right));
        assert_eq!(Action::TopHalf.as_half(), Some(Half::Top));
        assert_eq!(Action::BottomHalf.as_half(), Some(Half::Bottom));
        assert_eq!(Action::CenterHalf.as_half(), None);
        assert_eq!(Action::Maximize.as_half(), None);
    }
}
