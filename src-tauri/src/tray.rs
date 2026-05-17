use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Display-mode items
    let mode_full = MenuItem::with_id(app, "mode_full", "Show full window", true, None::<&str>)?;
    let mode_appbar = MenuItem::with_id(
        app,
        "mode_appbar",
        "Show compact (AppBar)",
        true,
        None::<&str>,
    )?;
    let mode_float = MenuItem::with_id(
        app,
        "mode_float",
        "Show compact (floating)",
        true,
        None::<&str>,
    )?;
    let mode_tray = MenuItem::with_id(app, "mode_tray", "Hide all (tray-only)", true, None::<&str>)?;
    let sep_modes = PredefinedMenuItem::separator(app)?;

    // Position submenu (legacy — applies to full window)
    let pos_tl = MenuItem::with_id(app, "pos_tl", "Top-Left", true, None::<&str>)?;
    let pos_tr = MenuItem::with_id(app, "pos_tr", "Top-Right", true, None::<&str>)?;
    let pos_bl = MenuItem::with_id(app, "pos_bl", "Bottom-Left", true, None::<&str>)?;
    let pos_br = MenuItem::with_id(app, "pos_br", "Bottom-Right", true, None::<&str>)?;
    let position_menu =
        Submenu::with_items(app, "Position (full)", true, &[&pos_tl, &pos_tr, &pos_bl, &pos_br])?;

    let always_on_top =
        MenuItem::with_id(app, "always_on_top", "Always on Top", true, None::<&str>)?;
    let retry_sensors =
        MenuItem::with_id(app, "retry_sensors", "Retry sensors", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let about = MenuItem::with_id(app, "about", "About SysMonitor", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &mode_full,
            &mode_appbar,
            &mode_float,
            &mode_tray,
            &sep_modes,
            &position_menu,
            &always_on_top,
            &retry_sensors,
            &sep2,
            &about,
            &sep3,
            &quit,
        ],
    )?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("app should have a default icon");

    TrayIconBuilder::new()
        .tooltip("SysMonitor - PC Monitoring Widget")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                // Display-mode switches — lib.rs listens for `set-mode`
                "mode_full" => {
                    app.emit("set-mode", "full").ok();
                }
                "mode_appbar" => {
                    app.emit("set-mode", "compact_appbar").ok();
                }
                "mode_float" => {
                    app.emit("set-mode", "compact_float").ok();
                }
                "mode_tray" => {
                    app.emit("set-mode", "tray_only").ok();
                }
                "retry_sensors" => {
                    app.emit("retry-sensors", ()).ok();
                }
                "quit" => {
                    app.exit(0);
                }
                "about" => {
                    if let Some(w) = app.get_webview_window("main") {
                        w.show().ok();
                        w.set_focus().ok();
                    }
                    app.emit("show-about", ()).ok();
                }
                // Legacy position presets — affect the main (full) window
                "pos_tl" => {
                    if let Some(window) = app.get_webview_window("main") {
                        use tauri::LogicalPosition;
                        window
                            .set_position(tauri::Position::Logical(LogicalPosition::new(10.0, 10.0)))
                            .ok();
                    }
                }
                "pos_tr" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if let Ok(Some(m)) = window.current_monitor() {
                            let size = m.size();
                            let scale = m.scale_factor();
                            let x = (size.width as f64 / scale) - 430.0;
                            use tauri::LogicalPosition;
                            window
                                .set_position(tauri::Position::Logical(LogicalPosition::new(x, 10.0)))
                                .ok();
                        }
                    }
                }
                "pos_bl" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if let Ok(Some(m)) = window.current_monitor() {
                            let size = m.size();
                            let scale = m.scale_factor();
                            let y = (size.height as f64 / scale) - 450.0 - 48.0;
                            use tauri::LogicalPosition;
                            window
                                .set_position(tauri::Position::Logical(LogicalPosition::new(10.0, y)))
                                .ok();
                        }
                    }
                }
                "pos_br" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if let Ok(Some(m)) = window.current_monitor() {
                            let size = m.size();
                            let scale = m.scale_factor();
                            let x = (size.width as f64 / scale) - 430.0;
                            let y = (size.height as f64 / scale) - 450.0 - 48.0;
                            use tauri::LogicalPosition;
                            window
                                .set_position(tauri::Position::Logical(LogicalPosition::new(x, y)))
                                .ok();
                        }
                    }
                }
                "always_on_top" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if let Ok(current) = window.is_always_on_top() {
                            window.set_always_on_top(!current).ok();
                        }
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                // Left-click cycles back to the last visible window
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        window.set_focus().ok();
                        return;
                    }
                }
                if let Some(window) = app.get_webview_window("compact") {
                    if !window.is_visible().unwrap_or(false) {
                        window.show().ok();
                    }
                    window.set_focus().ok();
                }
            }
        })
        .build(app)?;

    Ok(())
}
