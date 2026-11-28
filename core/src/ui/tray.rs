//! System tray icon: keeps the app reachable while the window is hidden.
//!
//! The icon must be created on the thread that runs the window's event loop (the UI
//! thread). Events are read with `poll()` from the UI's 250 ms tick.

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// What the user asked for from the tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    AddUrl,
    PauseAll,
    ResumeAll,
    Exit,
}

pub struct Tray {
    icon: TrayIcon,
    show: MenuId,
    add: MenuId,
    pause: MenuId,
    resume: MenuId,
    exit: MenuId,
    last_tooltip: String,
}

fn load_icon() -> Option<Icon> {
    let img = image::load_from_memory_with_format(include_bytes!("../../assets/icons/logo.png"), image::ImageFormat::Png)
        .map_err(|e| tracing::warn!("Tray icon image failed to load: {}", e))
        .ok()?
        .resize(32, 32, image::imageops::FilterType::Lanczos3)
        .into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

impl Tray {
    /// Creates the tray icon. Returns None if the shell refuses it (the app keeps working).
    pub fn new() -> Option<Self> {
        let show = MenuItem::new("Show 1nternet Downloader", true, None);
        let add = MenuItem::new("Add URL…", true, None);
        let resume = MenuItem::new("Resume all", true, None);
        let pause = MenuItem::new("Pause all", true, None);
        let exit = MenuItem::new("Exit", true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &show,
            &PredefinedMenuItem::separator(),
            &add,
            &resume,
            &pause,
            &PredefinedMenuItem::separator(),
            &exit,
        ])
        .ok()?;

        let icon = TrayIconBuilder::new()
            .with_tooltip("1nternet Downloader")
            .with_icon(load_icon()?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| tracing::warn!("System tray icon unavailable: {}", e))
            .ok()?;

        Some(Self {
            icon,
            show: show.id().clone(),
            add: add.id().clone(),
            pause: pause.id().clone(),
            resume: resume.id().clone(),
            exit: exit.id().clone(),
            last_tooltip: String::new(),
        })
    }

    /// Drains pending tray clicks and menu choices.
    pub fn poll(&self) -> Vec<TrayAction> {
        let mut out = Vec::new();
        while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
            match ev {
                TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }
                | TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => out.push(TrayAction::Show),
                _ => {}
            }
        }
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            let id = ev.id();
            if *id == self.show {
                out.push(TrayAction::Show);
            } else if *id == self.add {
                out.push(TrayAction::AddUrl);
            } else if *id == self.pause {
                out.push(TrayAction::PauseAll);
            } else if *id == self.resume {
                out.push(TrayAction::ResumeAll);
            } else if *id == self.exit {
                out.push(TrayAction::Exit);
            }
        }
        out.dedup();
        out
    }

    /// Updates the hover text (only when it changed).
    pub fn set_tooltip(&mut self, text: &str) {
        if self.last_tooltip != text {
            let _ = self.icon.set_tooltip(Some(text));
            self.last_tooltip = text.to_string();
        }
    }
}
