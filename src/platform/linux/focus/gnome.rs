//! GNOME Wayland focus detection backend
//!
//! Uses D-Bus to communicate with GNOME Shell.
//! TODO: Implement using zbus crate for proper D-Bus support.

use super::{FocusBackend, FocusedWindow};

pub struct GnomeFocusBackend {
    // TODO: D-Bus connection
}

impl GnomeFocusBackend {
    pub fn new() -> Option<Self> {
        // TODO: Implement GNOME D-Bus connection
        // Interface: org.gnome.Shell.Introspect (GNOME 45+)
        // Method: GetWindows() returns window list with focus info
        log::warn!("GNOME Wayland backend not yet implemented");
        None
    }
}

impl FocusBackend for GnomeFocusBackend {
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        // TODO: Query org.gnome.Shell.Introspect.GetWindows()
        // Filter for focused window
        None
    }

    fn name(&self) -> &'static str {
        "GNOME Wayland"
    }
}
