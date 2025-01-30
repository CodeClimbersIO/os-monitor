# Monitor

The monitor is a Rust application that runs on your computer and is responsible for monitoring your activities. It is specifically responsible for monitoring (but not recording) your window, mouse and keyboard activity.
Architecture supports multiple platforms.

Implemented platforms:

[x] macOS  
[ ] Windows  
[ ] Linux  

## Example Usage
Here's an example of how to use the monitor as a library by registering event callbacks:

```
   let monitor = Monitor::new();

   // Register event callbacks
   monitor.register_keyboard_callback(Box::new(on_keyboard_events));
   monitor.register_mouse_callback(Box::new(on_mouse_events));
   monitor.register_window_callback(Box::new(on_window_event));

   initialize_monitor(Arc::new(monitor)).expect("Failed to initialize monitor");
   loop {
      detect_changes().expect("Failed to detect changes");
      std::thread::sleep(std::time::Duration::from_secs(1));
   }
```

On first run on macOS, you'll need to grant accessibility permissions to the application. This is required to monitor window focus and input events.

### Building and Running
   ```bash
   cargo build
   cargo run
   ```

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- For macOS:
  - Xcode Command Line Tools

## Architecture
See [architecture.md](architecture.md) for more information.

## Security and Privacy

- The monitor only tracks event metadata, not content
- No keystrokes are recorded, only key codes
- Window titles and application names are captured for context
- All data processing happens locally

## Development Guidelines

### Adding OS Support

To add support for a new OS platform:

1. Create new platform-specific module in `src/platform/`
2. Implement native bindings in `bindings/`
3. Implement required traits and functions
4. Update conditional compilation flags


### Other notes
Brought over from the original repo: https://github.com/CodeClimbersIO/app-codeclimbers