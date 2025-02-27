use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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

pub trait EventCallback: Send + Sync {
    fn on_mouse_events(&self, has_activity: bool);
    fn on_keyboard_events(&self, has_activity: bool);
    fn on_window_event(&self, event: WindowEvent);
}
pub struct Monitor {
    mouse_callbacks: Mutex<Vec<Box<dyn Fn(bool) + Send + Sync>>>,
    keyboard_callbacks: Mutex<Vec<Box<dyn Fn(bool) + Send + Sync>>>,
    window_callbacks: Mutex<Vec<Box<dyn Fn(WindowEvent) + Send + Sync>>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            mouse_callbacks: Mutex::new(Vec::new()),
            keyboard_callbacks: Mutex::new(Vec::new()),
            window_callbacks: Mutex::new(Vec::new()),
        }
    }

    pub fn register_keyboard_callback(&self, callback: Box<dyn Fn(bool) + Send + Sync>) {
        self.keyboard_callbacks.lock().unwrap().push(callback);
    }

    pub fn register_mouse_callback(&self, callback: Box<dyn Fn(bool) + Send + Sync>) {
        self.mouse_callbacks.lock().unwrap().push(callback);
    }

    pub fn register_window_callback(&self, callback: Box<dyn Fn(WindowEvent) + Send + Sync>) {
        self.window_callbacks.lock().unwrap().push(callback);
    }
}

impl EventCallback for Monitor {
    fn on_mouse_events(&self, has_activity: bool) {
        let mut callbacks = self.mouse_callbacks.lock().unwrap();
        for callback in callbacks.iter_mut() {
            callback(has_activity);
        }
    }

    fn on_keyboard_events(&self, has_activity: bool) {
        let mut callbacks = self.keyboard_callbacks.lock().unwrap();
        for callback in callbacks.iter_mut() {
            callback(has_activity);
        }
    }

    fn on_window_event(&self, event: WindowEvent) {
        let mut callbacks = self.window_callbacks.lock().unwrap();
        for callback in callbacks.iter_mut() {
            callback(event.clone());
        }
    }
}
