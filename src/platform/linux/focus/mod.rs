//! Focus detection backends for different Linux display servers/compositors
//!
//! This module abstracts window focus detection so we can support multiple
//! backends (X11, Hyprland, GNOME, KDE, etc.) without changing the rest of the codebase.
//!
//! Consumers only see:
//! - `FocusedWindow` struct
//! - `FocusBackend` trait
//! - `create_focus_backend()` factory function
//!
//! The specific backend implementations are private.

mod x11;
mod hyprland;
mod gnome;
mod kde;

use std::env;

/// Information about the currently focused window
#[derive(Debug, Clone)]
pub struct FocusedWindow {
    /// Window/surface identifier (X11 window ID, Hyprland address, etc.)
    pub id: u64,
    /// Application name/class (used for blocking)
    pub app_name: String,
    /// Window title
    pub title: String,
}

/// Trait for focus detection backends
pub trait FocusBackend: Send + Sync {
    /// Get the currently focused window, if any
    fn get_focused_window(&self) -> Option<FocusedWindow>;

    /// Backend name for logging
    fn name(&self) -> &'static str;
}

/// Detected display server/compositor type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Hyprland,
    GnomeWayland,
    KdeWayland,
    Unknown,
}

/// Detect which display server/compositor is running
pub fn detect_display_server() -> DisplayServer {
    // Check for Hyprland first (it also sets WAYLAND_DISPLAY)
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return DisplayServer::Hyprland;
    }

    // Check for GNOME on Wayland
    if env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_uppercase().contains("GNOME") {
                return DisplayServer::GnomeWayland;
            }
        }
    }

    // Check for KDE on Wayland
    if env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_uppercase().contains("KDE") {
                return DisplayServer::KdeWayland;
            }
        }
    }

    // Check for pure X11 (no Wayland)
    if env::var("DISPLAY").is_ok() && env::var("WAYLAND_DISPLAY").is_err() {
        return DisplayServer::X11;
    }

    // XWayland fallback: both DISPLAY and WAYLAND_DISPLAY are set
    // but we don't have a native backend for this compositor
    if env::var("DISPLAY").is_ok() {
        log::info!("Wayland detected but no native backend, using XWayland fallback");
        return DisplayServer::X11;
    }

    DisplayServer::Unknown
}

/// Create the appropriate focus backend for the current environment
pub fn create_focus_backend() -> Option<Box<dyn FocusBackend>> {
    let display_server = detect_display_server();
    log::info!("Detected display server: {:?}", display_server);

    match display_server {
        DisplayServer::X11 => {
            Some(Box::new(x11::X11FocusBackend::new()?))
        }
        DisplayServer::Hyprland => {
            match hyprland::HyprlandFocusBackend::new() {
                Some(backend) => Some(Box::new(backend)),
                None => {
                    log::warn!("Hyprland backend failed, falling back to XWayland");
                    Some(Box::new(x11::X11FocusBackend::new()?))
                }
            }
        }
        DisplayServer::GnomeWayland => {
            match gnome::GnomeFocusBackend::new() {
                Some(backend) => Some(Box::new(backend)),
                None => {
                    log::warn!("GNOME Wayland backend failed, falling back to XWayland");
                    Some(Box::new(x11::X11FocusBackend::new()?))
                }
            }
        }
        DisplayServer::KdeWayland => {
            match kde::KdeFocusBackend::new() {
                Some(backend) => Some(Box::new(backend)),
                None => {
                    log::warn!("KDE Wayland backend failed, falling back to XWayland");
                    Some(Box::new(x11::X11FocusBackend::new()?))
                }
            }
        }
        DisplayServer::Unknown => {
            log::error!("Could not detect display server");
            None
        }
    }
}
