use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Platform {
    Mac,
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseEventType {
    Move,
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
    MiddleDown,
    MiddleUp,
    Scroll,
}

impl TryFrom<i32> for MouseEventType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseEventType::Move),
            1 => Ok(MouseEventType::LeftDown),
            2 => Ok(MouseEventType::LeftUp),
            3 => Ok(MouseEventType::RightDown),
            4 => Ok(MouseEventType::RightUp),
            5 => Ok(MouseEventType::MiddleDown),
            6 => Ok(MouseEventType::MiddleUp),
            7 => Ok(MouseEventType::Scroll),
            _ => Err("Invalid mouse event type"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WindowEventType {
    Focused,
    TitleChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEvent {
    pub app_name: String,
    pub window_title: String,
    pub bundle_id: Option<String>,
    pub url: Option<String>,
    pub platform: Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedApp {
    pub app_name: String,
    pub app_external_id: String,
    pub is_site: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedAppEvent {
    pub blocked_apps: Vec<BlockedApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    Mouse(bool),
    Keyboard(bool),
    Window(WindowEvent),
    AppBlocked(BlockedAppEvent),
}

pub struct Monitor {
    event_sender: Sender<AppEvent>,
}

impl Monitor {
    pub fn new() -> Self {
        // A capacity of 100 should be more than enough for most use cases
        let (sender, _) = broadcast::channel(100);

        Self {
            event_sender: sender,
        }
    }

    /// Get a new receiver to subscribe to events
    pub fn subscribe(&self) -> Receiver<AppEvent> {
        self.event_sender.subscribe()
    }

    pub fn send_mouse_event(&self, has_activity: bool) {
        let _ = self.event_sender.send(AppEvent::Mouse(has_activity));
    }

    pub fn send_keyboard_event(&self, has_activity: bool) {
        let _ = self.event_sender.send(AppEvent::Keyboard(has_activity));
    }

    pub fn send_window_event(&self, event: WindowEvent) {
        let _ = self.event_sender.send(AppEvent::Window(event));
    }

    pub fn send_app_blocked_event(&self, event: BlockedAppEvent) {
        let _ = self.event_sender.send(AppEvent::AppBlocked(event));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_multiple_receivers() {
        // Create a monitor
        let monitor = Monitor::new();

        // Create three receivers
        let mut receiver1 = monitor.subscribe();
        let mut receiver2 = monitor.subscribe();
        let mut receiver3 = monitor.subscribe();

        // Send a mouse event
        monitor.send_mouse_event(true);

        // Test that all receivers get the event
        match receiver1.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => assert!(activity),
            _ => panic!("Receiver 1 did not get the correct event"),
        }

        match receiver2.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => assert!(activity),
            _ => panic!("Receiver 2 did not get the correct event"),
        }

        match receiver3.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => assert!(activity),
            _ => panic!("Receiver 3 did not get the correct event"),
        }
    }

    #[test]
    fn test_late_subscriber() {
        // Create a monitor
        let monitor = Monitor::new();

        // Create initial receiver
        let mut receiver1 = monitor.subscribe();

        // Send first event
        monitor.send_mouse_event(true);

        // Create a late subscriber
        let mut receiver2 = monitor.subscribe();

        // Send second event
        monitor.send_keyboard_event(true);

        // First receiver gets both events
        match receiver1.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => assert!(activity),
            _ => panic!("Receiver 1 did not get the mouse event"),
        }

        match receiver1.blocking_recv() {
            Ok(AppEvent::Keyboard(activity)) => assert!(activity),
            _ => panic!("Receiver 1 did not get the keyboard event"),
        }

        // Second receiver only gets the keyboard event (missed the mouse event)
        match receiver2.blocking_recv() {
            Ok(AppEvent::Keyboard(activity)) => assert!(activity),
            _ => panic!("Receiver 2 did not get the keyboard event"),
        }
    }

    #[test]
    fn test_multiple_threads() {
        // Create a monitor
        let monitor = Monitor::new();

        // Create receivers for different threads
        let mut receiver1 = monitor.subscribe();
        let mut receiver2 = monitor.subscribe();
        let mut receiver3 = monitor.subscribe();

        // Spawn threads to listen for events
        let handle1 = thread::spawn(move || match receiver1.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => activity,
            _ => false,
        });

        let handle2 = thread::spawn(move || match receiver2.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => activity,
            _ => false,
        });

        let handle3 = thread::spawn(move || match receiver3.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => activity,
            _ => false,
        });

        // Wait a moment to ensure threads are ready
        thread::sleep(std::time::Duration::from_millis(100));

        // Send event
        monitor.send_mouse_event(true);

        // Check results from all threads
        assert!(handle1.join().unwrap());
        assert!(handle2.join().unwrap());
        assert!(handle3.join().unwrap());
    }

    #[test]
    fn test_all_event_types() {
        // Create a monitor
        let monitor = Monitor::new();

        // Create receivers
        let mut receiver1 = monitor.subscribe();
        let mut receiver2 = monitor.subscribe();

        // Send different types of events
        monitor.send_mouse_event(true);
        monitor.send_keyboard_event(false);

        let window_event = WindowEvent {
            app_name: "Test App".to_string(),
            window_title: "Test Window".to_string(),
            bundle_id: Some("com.test.app".to_string()),
            url: None,
            platform: Platform::Mac,
        };
        monitor.send_window_event(window_event.clone());

        let blocked_app = BlockedApp {
            app_name: "Block Test".to_string(),
            app_external_id: "com.block.test".to_string(),
            is_site: false,
        };
        monitor.send_app_blocked_event(BlockedAppEvent {
            blocked_apps: vec![blocked_app.clone()],
        });

        // Test first receiver gets all events
        match receiver1.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => assert!(activity),
            _ => panic!("Receiver 1 did not get the mouse event"),
        }

        match receiver1.blocking_recv() {
            Ok(AppEvent::Keyboard(activity)) => assert!(!activity),
            _ => panic!("Receiver 1 did not get the keyboard event"),
        }

        match receiver1.blocking_recv() {
            Ok(AppEvent::Window(event)) => {
                assert_eq!(event.app_name, "Test App");
                assert_eq!(event.window_title, "Test Window");
            }
            _ => panic!("Receiver 1 did not get the window event"),
        }

        match receiver1.blocking_recv() {
            Ok(AppEvent::AppBlocked(event)) => {
                assert_eq!(event.blocked_apps.len(), 1);
                assert_eq!(event.blocked_apps[0].app_name, "Block Test");
            }
            _ => panic!("Receiver 1 did not get the app blocked event"),
        }

        // Test second receiver also gets all events
        match receiver2.blocking_recv() {
            Ok(AppEvent::Mouse(activity)) => assert!(activity),
            _ => panic!("Receiver 2 did not get the mouse event"),
        }

        match receiver2.blocking_recv() {
            Ok(AppEvent::Keyboard(activity)) => assert!(!activity),
            _ => panic!("Receiver 2 did not get the keyboard event"),
        }

        match receiver2.blocking_recv() {
            Ok(AppEvent::Window(event)) => {
                assert_eq!(event.app_name, "Test App");
                assert_eq!(event.window_title, "Test Window");
            }
            _ => panic!("Receiver 2 did not get the window event"),
        }

        match receiver2.blocking_recv() {
            Ok(AppEvent::AppBlocked(event)) => {
                assert_eq!(event.blocked_apps.len(), 1);
                assert_eq!(event.blocked_apps[0].app_name, "Block Test");
            }
            _ => panic!("Receiver 2 did not get the app blocked event"),
        }
    }
}
