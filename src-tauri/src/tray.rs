use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    let pos_tl = MenuItem::with_id(app, "pos_tl", "Top-Left", true, None::<&str>)?;
    let pos_tr = MenuItem::with_id(app, "pos_tr", "Top-Right", true, None::<&str>)?;
    let pos_bl = MenuItem::with_id(app, "pos_bl", "Bottom-Left", true, None::<&str>)?;
    let pos_br = MenuItem::with_id(app, "pos_br", "Bottom-Right", true, None::<&str>)?;
    let position_menu = Submenu::with_items(
        app,
        "Position",
        true,
        &[&pos_tl, &pos_tr, &pos_bl, &pos_br],
    )?;

    let always_on_top =
        MenuItem::with_id(app, "always_on_top", "Always on Top", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let about = MenuItem::with_id(app, "about", "About SysMonitor", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &sep1,
            &position_menu,
            &always_on_top,
            &sep2,
            &about,
            &sep3,
            &quit,
        ],
    )?;

    // Use the app's default icon for the tray
    let icon = app.default_window_icon().cloned().expect("app should have a default icon");

    TrayIconBuilder::new()
        .tooltip("SysMonitor - PC Monitoring Widget")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app, event| {
            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => return,
            };
            match event.id.as_ref() {
                "show" => {
                    window.show().ok();
                    window.set_focus().ok();
                }
                "hide" => {
                    window.hide().ok();
                }
                "quit" => {
                    app.exit(0);
                }
                "about" => {
                    window.show().ok();
                    window.set_focus().ok();
                    app.emit("show-about", ()).ok();
                }
                "pos_tl" => {
                    use tauri::LogicalPosition;
                    window
                        .set_position(tauri::Position::Logical(LogicalPosition::new(10.0, 10.0)))
                        .ok();
                }
                "pos_tr" => {
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
                "pos_bl" => {
                    if let Ok(Some(m)) = window.current_monitor() {
                        let size = m.size();
                        let scale = m.scale_factor();
                        let y = (size.height as f64 / scale) - 390.0 - 48.0;
                        use tauri::LogicalPosition;
                        window
                            .set_position(tauri::Position::Logical(LogicalPosition::new(10.0, y)))
                            .ok();
                    }
                }
                "pos_br" => {
                    if let Ok(Some(m)) = window.current_monitor() {
                        let size = m.size();
                        let scale = m.scale_factor();
                        let x = (size.width as f64 / scale) - 430.0;
                        let y = (size.height as f64 / scale) - 390.0 - 48.0;
                        use tauri::LogicalPosition;
                        window
                            .set_position(tauri::Position::Logical(LogicalPosition::new(x, y)))
                            .ok();
                    }
                }
                "always_on_top" => {
                    if let Ok(current) = window.is_always_on_top() {
                        window.set_always_on_top(!current).ok();
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
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
        })
        .build(app)?;

    Ok(())
}
