//! Pure functions for computing widget corner positions on the primary monitor.
//!
//! Used by the compact bar's "Position" preset menu to snap to TL/TR/BL/BR.
//! Bottom positions subtract a taskbar offset so the widget sits *above* the
//! Windows taskbar rather than behind it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "top-left" | "topleft" | "tl" => Some(Corner::TopLeft),
            "top-right" | "topright" | "tr" => Some(Corner::TopRight),
            "bottom-left" | "bottomleft" | "bl" => Some(Corner::BottomLeft),
            "bottom-right" | "bottomright" | "br" => Some(Corner::BottomRight),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Corner::TopLeft => "top-left",
            Corner::TopRight => "top-right",
            Corner::BottomLeft => "bottom-left",
            Corner::BottomRight => "bottom-right",
        }
    }
}

pub const MARGIN: i32 = 4;

/// Compute logical-pixel (x, y) for the widget's top-left corner.
///
/// `taskbar_offset` is subtracted from the bottom for `BottomLeft`/`BottomRight`
/// so the widget clears the Windows taskbar. Pass 0 if no taskbar is present
/// (e.g. taskbar auto-hide) or for top-edge corners.
pub fn compute_corner_position(
    monitor_width: i32,
    monitor_height: i32,
    widget_width: i32,
    widget_height: i32,
    corner: Corner,
    taskbar_offset: i32,
) -> (i32, i32) {
    match corner {
        Corner::TopLeft => (MARGIN, MARGIN),
        Corner::TopRight => (monitor_width - widget_width - MARGIN, MARGIN),
        Corner::BottomLeft => (
            MARGIN,
            monitor_height - widget_height - taskbar_offset - MARGIN,
        ),
        Corner::BottomRight => (
            monitor_width - widget_width - MARGIN,
            monitor_height - widget_height - taskbar_offset - MARGIN,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1920×1080 monitor with a 360×30 widget and a 48-px taskbar.
    const W: i32 = 1920;
    const H: i32 = 1080;
    const WW: i32 = 360;
    const WH: i32 = 30;
    const TB: i32 = 48;

    #[test]
    fn top_left_uses_margin_only() {
        let (x, y) = compute_corner_position(W, H, WW, WH, Corner::TopLeft, TB);
        assert_eq!((x, y), (MARGIN, MARGIN));
    }

    #[test]
    fn top_right_subtracts_widget_width_and_margin() {
        let (x, y) = compute_corner_position(W, H, WW, WH, Corner::TopRight, TB);
        assert_eq!((x, y), (W - WW - MARGIN, MARGIN));
    }

    #[test]
    fn bottom_left_clears_taskbar() {
        let (x, y) = compute_corner_position(W, H, WW, WH, Corner::BottomLeft, TB);
        assert_eq!(x, MARGIN);
        assert_eq!(y, H - WH - TB - MARGIN);
        // Sanity: widget bottom edge is above the taskbar
        assert!(y + WH <= H - TB);
    }

    #[test]
    fn bottom_right_clears_taskbar() {
        let (x, y) = compute_corner_position(W, H, WW, WH, Corner::BottomRight, TB);
        assert_eq!(x, W - WW - MARGIN);
        assert_eq!(y, H - WH - TB - MARGIN);
    }

    #[test]
    fn bottom_with_zero_taskbar_still_fits() {
        let (_, y) = compute_corner_position(W, H, WW, WH, Corner::BottomLeft, 0);
        assert_eq!(y, H - WH - MARGIN);
        assert!(y + WH <= H);
    }

    #[test]
    fn corner_from_str_accepts_canonical_names() {
        assert_eq!(Corner::from_str("top-left"), Some(Corner::TopLeft));
        assert_eq!(Corner::from_str("top-right"), Some(Corner::TopRight));
        assert_eq!(Corner::from_str("bottom-left"), Some(Corner::BottomLeft));
        assert_eq!(Corner::from_str("bottom-right"), Some(Corner::BottomRight));
    }

    #[test]
    fn corner_from_str_accepts_aliases_and_case() {
        assert_eq!(Corner::from_str("TL"), Some(Corner::TopLeft));
        assert_eq!(Corner::from_str("BottomLeft"), Some(Corner::BottomLeft));
        assert_eq!(Corner::from_str("BR"), Some(Corner::BottomRight));
    }

    #[test]
    fn corner_from_str_rejects_garbage() {
        assert_eq!(Corner::from_str(""), None);
        assert_eq!(Corner::from_str("middle"), None);
        assert_eq!(Corner::from_str("topish-left"), None);
    }

    #[test]
    fn corner_as_str_roundtrips_through_from_str() {
        for c in [Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight] {
            assert_eq!(Corner::from_str(c.as_str()), Some(c));
        }
    }
}
