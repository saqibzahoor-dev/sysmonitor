//! Defines the right-click context menu for the compact bar window.
//! Pure data — kept separate from the Tauri menu-building code so the
//! layout can be unit-tested.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    /// (id, label) — clicking emits the event-id back to Rust
    Item(&'static str, &'static str),
    Separator,
}

/// Items shown when right-clicking the compact bar, in display order.
/// IDs match the tray menu's IDs where overlap exists so the same Rust
/// handler in tray.rs / lib.rs `set-mode`/`set-compact-position`/etc
/// can fire from either source.
pub fn compact_menu_layout() -> Vec<MenuEntry> {
    vec![
        MenuEntry::Item("cpos_tl", "↖  Top-Left"),
        MenuEntry::Item("cpos_tr", "↗  Top-Right"),
        MenuEntry::Item("cpos_bl", "↙  Bottom-Left"),
        MenuEntry::Item("cpos_br", "↘  Bottom-Right"),
        MenuEntry::Separator,
        MenuEntry::Item("mode_full", "▢  Open Full Window"),
        MenuEntry::Item("always_on_top", "⊞  Always on Top"),
        MenuEntry::Item("retry_sensors", "↻  Retry Sensors"),
        MenuEntry::Separator,
        MenuEntry::Item("quit", "✕  Quit"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_starts_with_four_position_presets() {
        let items = compact_menu_layout();
        assert!(matches!(&items[0], MenuEntry::Item("cpos_tl", _)));
        assert!(matches!(&items[1], MenuEntry::Item("cpos_tr", _)));
        assert!(matches!(&items[2], MenuEntry::Item("cpos_bl", _)));
        assert!(matches!(&items[3], MenuEntry::Item("cpos_br", _)));
    }

    #[test]
    fn layout_has_separator_after_positions() {
        let items = compact_menu_layout();
        assert_eq!(items[4], MenuEntry::Separator);
    }

    #[test]
    fn layout_ends_with_quit() {
        let items = compact_menu_layout();
        let last = items.last().unwrap();
        assert!(matches!(last, MenuEntry::Item("quit", _)));
    }

    #[test]
    fn layout_contains_open_full_window() {
        let items = compact_menu_layout();
        assert!(items.iter().any(|e| matches!(e, MenuEntry::Item("mode_full", _))));
    }

    #[test]
    fn layout_contains_retry_sensors() {
        let items = compact_menu_layout();
        assert!(items.iter().any(|e| matches!(e, MenuEntry::Item("retry_sensors", _))));
    }

    #[test]
    fn layout_has_exactly_two_separators() {
        let items = compact_menu_layout();
        let seps = items.iter().filter(|e| matches!(e, MenuEntry::Separator)).count();
        assert_eq!(seps, 2);
    }

    #[test]
    fn ids_for_position_presets_match_tray() {
        // Tray uses cpos_tl/tr/bl/br for compact-position; we reuse.
        let items = compact_menu_layout();
        let ids: Vec<&str> = items
            .iter()
            .filter_map(|e| match e {
                MenuEntry::Item(id, _) => Some(*id),
                _ => None,
            })
            .collect();
        assert!(ids.contains(&"cpos_tl"));
        assert!(ids.contains(&"cpos_tr"));
        assert!(ids.contains(&"cpos_bl"));
        assert!(ids.contains(&"cpos_br"));
    }
}
