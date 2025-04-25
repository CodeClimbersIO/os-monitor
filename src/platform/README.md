# Platform Module Interface

This directory contains platform-specific implementations for the OS monitor functionality. Currently, implementations exist for macOS and Windows. This README outlines the functions that need to be implemented for a Linux version.

## Core Interface Functions

Each platform implementation must provide the following functions:

### `platform_start_monitoring`

Starts monitoring user activity mouse, keyboard and focused windows.

**Implementation requirements:**
- Register callbacks for mouse and keyboard events

### `platform_detect_changes() -> Result<(), MonitorError>`

This is the primary workhorse driver of functionality for the monitor. This is called once per second. Each execution emits events about window activity and handles blocking when turned on. Every 30 seconds, if there were keyboard or mouse events, we send that information as an event. 

**Implementation requirements:**
- Check for newly focused windows
- Gather window information (title, application name, bundle ID/identifier)
- Monitor URLs in web browsers
- Trigger appropriate callbacks when blocked content is detected
- when a site is blocked, redirect the site and send a blocked event
- when an app is blocked, close the app and send a blocked event
- Send buffered activity events periodically
- Will be called from a background thread as we don't want to eat up the main thread and make the ui unresponsive

### `platform_start_blocking`

Starts blocking specified applications and/or websites. Doesn't perform any of the actual blocking--that is performed by detect_changes()--just flips on the state values that are used to determine whether or not to skip the blocking steps when detecting the currently focused app. If this is turned on, the blocking checks are performed. 

Look at the `macos.rs` implementation for details on applications that should be exceptions from the blocklist. These are things like system applications that should not ever be closed. It also adds browser apps to the list of exceptions if we're providing an allowlist and one of the blocked_apps is a website

**Parameters:**
- `blocked_apps`: List of applications/websites to block (or allow if in allowlist mode)
- `redirect_url`: URL to redirect to when a blocked website is accessed
- `blocklist_mode`: If true, block listed items; if false, block everything except listed items (allowlist mode)

**Implementation requirements:**
- Convert application identifiers to platform-specific format
- Set up hooks to detect when blocked applications are launched
- For web browsers, implement URL filtering/redirecting
- Return true if blocking was successfully enabled

### `platform_stop_blocking()`

Stops all application/website blocking. Turns off the state value for performing blocking so that the blocking actions are skipped.

**Implementation requirements:**
- Turns off the state values for performing blocking

### `platform_has_accessibility_permissions() -> bool`

Checks if the application has the necessary accessibility permissions. This may/may not translate to other operating systems. 

**Implementation requirements:**
- Return true if accessibility permissions are granted, false otherwise

### `platform_request_accessibility_permissions() -> bool`

Requests accessibility permissions from the user. This may/maynot translate to other operating systems

**Implementation requirements:**
- Show appropriate dialog or instructions for enabling permissions
- Return true if permissions were granted, false otherwise

### `platform_get_application_icon_data(bundle_id: &str) -> Option<String>`

Retrieves icon data for an application. Given the application id to the system, returns back the app icon as a base64 encoded string.

**Parameters:**
- `bundle_id`: Application identifier

**Implementation requirements:**
- Locate application icon based on identifier
- Return encoded icon data (Base64 or other appropriate format)
- Return None if icon cannot be found

### `platform_run_loop_cycle()`

Runs a single cycle of the event processing loop. This is required on MacOS for any visual changes that are being made (say with typewriter mode). Called at the same cadence as detect_changes(). May or may not be required on other operating systems

**Implementation requirements:**
- Process pending events in the platform's event queue
- On macOS, has to be called from the main thread

### `platform_create_typewriter_window(opacity: f64)`

Creates a visual "typewriter" black overlay window with specified opacity. This overlay is fullscreen and goes directly behind the frontmost window, creating the effect of unfocusing the rest of the computer content. 

**Parameters:**
- `opacity`: Window opacity (0.0 to 1.0)

**Implementation requirements:**
- Create a window that stays on top of other windows besides the frontmost
- Set appropriate transparency/opacity

### `platform_sync_typewriter_window_order()`

Ensures the typewriter window stays behind the frontmost window as the user changes focus (i.e. clicks on a new app and brings it into the foreground as the new thing they are working on)

**Implementation requirements:**
- Update window z-order to maintain visibility

### `platform_remove_typewriter_window()`

Removes the typewriter window.

**Implementation requirements:**
- Destroy the window and clean up resources

## Implementation Notes

## Testing

Still working on better ways to test this. Right now, been running main.rs primarily as it runs all of the different pieces of functionality as they have been designed. 