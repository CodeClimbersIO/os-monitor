# Monitor

The monitor is a Rust application that runs on your computer and is responsible for monitoring your activities. It is specifically responsible for monitoring (but not recording) your window, mouse and keyboard activity.
Architecture is intended to support multiple platforms.

Implemented platforms:

- [x] macOS
- [ ] Windows
- [ ] Linux

## Supported functionality
Refer to [src/platform.README.md](src/platform/README.md) for a list of supported functions and their functionality

## Example Usage
Refer to `main.rs` for how the different

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
- Window titles, application names, and browser urls are captured for context
- All data processing happens locally

## Development Guidelines

### Adding OS Support

To add support for a new OS platform:

1. Create new platform-specific module in `src/platform/`
2. Implement native bindings in `bindings/`
3. Implement required traits and functions
4. Update conditional compilation flags
5. Refer to [src/platform.README.md](src/platform/README.md) for functionality to mimic


### Other notes
Brought over from the original repo: https://github.com/CodeClimbersIO/app-codeclimbers
