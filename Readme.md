# Monitor

The monitor is a Rust application that runs on your computer and is responsible for monitoring your activities. It is specifically responsible for monitoring (but not recording) your window, mouse and keyboard activity.


## Getting Started

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- For macOS:
  - Xcode Command Line Tools
  - Accessibility permissions (will be prompted on first run)
- For Windows:
  - Visual Studio with C++ development tools
  - Windows SDK

### Building and Running

1. Clone the repository
2. Navigate to the monitor directory
3. Build the project:   ```bash
   cargo build   ```
4. Run the monitor:   ```bash
   cargo run   ```

On first run on macOS, you'll need to grant accessibility permissions to the application. This is required to monitor window focus and input events.

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