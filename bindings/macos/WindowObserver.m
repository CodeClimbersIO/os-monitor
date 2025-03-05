#import "WindowObserver.h"
#import "Application.h"
#import <ApplicationServices/ApplicationServices.h>

@interface WindowObserver ()
@property(nonatomic, strong) NSMutableDictionary *windowObservers;
@property(nonatomic) AXObserverRef currentAppObserver;
@property(nonatomic, strong) AccessibilityElement *currentAppElement;
@property(nonatomic, assign) WindowChangeCallback callback;
@property(nonatomic, assign) BOOL isObserving;
@property(nonatomic, strong) id appSwitchObserver;
@end

@implementation WindowObserver

+ (instancetype)sharedObserver {
  static WindowObserver *sharedInstance = nil;
  static dispatch_once_t onceToken;
  dispatch_once(&onceToken, ^{
    sharedInstance = [[WindowObserver alloc] init];
  });
  return sharedInstance;
}

- (instancetype)init {
  self = [super init];
  if (self) {
    _windowObservers = [NSMutableDictionary dictionary];
    _isObserving = NO;
  }
  return self;
}

- (void)dealloc {
  [self stopObserving];
}

// Callback for window title changes
static void windowTitleCallback(AXObserverRef observer, AXUIElementRef element,
                                CFStringRef notification, void *contextData) {
  WindowObserver *self = (__bridge WindowObserver *)contextData;
  [self handleWindowTitleChange:element];
}

// Callback for focused window changes
static void focusedWindowCallback(AXObserverRef observer,
                                  AXUIElementRef element,
                                  CFStringRef notification, void *contextData) {
  WindowObserver *self = (__bridge WindowObserver *)contextData;
  [self handleFocusedWindowChange:element];
}

- (void)handleWindowTitleChange:(AXUIElementRef)element {
  [self notifyWindowChange:element];
}

- (void)handleFocusedWindowChange:(AXUIElementRef)element {
  // Get the focused window
  AXUIElementRef focusedWindow = NULL;
  AXError error = AXUIElementCopyAttributeValue(
      element, kAXFocusedWindowAttribute, (CFTypeRef *)&focusedWindow);

  if (error != kAXErrorSuccess || focusedWindow == NULL) {
    NSLog(@"Error getting focused window: %d", error);
    return;
  }

  [self observeWindowTitleChanges:focusedWindow];
  [self notifyWindowChange:focusedWindow];

  CFRelease(focusedWindow);
}

- (void)observeWindowTitleChanges:(AXUIElementRef)window {
  // Create a unique identifier for this window
  NSValue *windowRef = [NSValue valueWithPointer:(void *)window];

  // Check if we're already observing this window
  if (![_windowObservers objectForKey:windowRef]) {
    // Create a new observer for this window
    pid_t pid;
    AXUIElementGetPid(window, &pid);

    AXObserverRef windowObserver = NULL;
    AXError error = AXObserverCreate(pid, windowTitleCallback, &windowObserver);

    if (error == kAXErrorSuccess && windowObserver != NULL) {
      // Register for title changes on this specific window
      error = AXObserverAddNotification(windowObserver, window,
                                        kAXTitleChangedNotification,
                                        (__bridge void *)self);

      if (error == kAXErrorSuccess) {
        // Add the observer to the run loop
        CFRunLoopAddSource(CFRunLoopGetCurrent(),
                           AXObserverGetRunLoopSource(windowObserver),
                           kCFRunLoopDefaultMode);

        // Store the observer
        [_windowObservers setObject:(__bridge id)(windowObserver)
                             forKey:windowRef];
      } else {
        NSLog(@"Error adding title notification to window: %d", error);
        CFRelease(windowObserver);
      }
    }
  }
}

- (void)notifyWindowChange:(AXUIElementRef)window {
  if (!_callback) {
    return;
  }

  // We'll use the AccessibilityElement and AppWindow classes for consistency
  pid_t pid;
  if (AXUIElementGetPid(window, &pid) != kAXErrorSuccess) {
    return;
  }

  NSRunningApplication *app =
      [NSRunningApplication runningApplicationWithProcessIdentifier:pid];
  if (!app) {
    return;
  }

  // Use our existing classes to get window info
  FocusedApp *focusedApp = [[FocusedApp alloc] initWithRunningApplication:app];
  WindowTitle *windowInfo = [focusedApp windowTitleStructWithWindow];

  if (windowInfo) {
    _callback(windowInfo->app_name, windowInfo->window_title,
              windowInfo->bundle_id, windowInfo->url);

    // Free the window info (our callback will have made copies if needed)
    free_window_title(windowInfo);
  }
}

- (BOOL)startObservingApp:(pid_t)pid {
  // Clean up previous app observer if it exists
  [self cleanupCurrentAppObserver];

  // Create a new observer for the application
  AXError error =
      AXObserverCreate(pid, focusedWindowCallback, &_currentAppObserver);
  if (error != kAXErrorSuccess) {
    NSLog(@"Error creating observer: %d", error);
    return NO;
  }

  // Create a reference to the application's UI element
  AXUIElementRef appElement = AXUIElementCreateApplication(pid);
  _currentAppElement =
      [[AccessibilityElement alloc] initWithAXUIElement:appElement];
  CFRelease(appElement); // AccessibilityElement retains it

  // Register for focused window changed notification
  BOOL success =
      [_currentAppElement addObserver:_currentAppObserver
                         notification:kAXFocusedWindowChangedNotification
                             callback:focusedWindowCallback
                             userData:(__bridge void *)self];

  if (!success) {
    [self cleanupCurrentAppObserver];
    return NO;
  }

  // Add the observer to the run loop
  CFRunLoopAddSource(CFRunLoopGetCurrent(),
                     AXObserverGetRunLoopSource(_currentAppObserver),
                     kCFRunLoopDefaultMode);

  // Trigger the callback once to observe the currently focused window
  focusedWindowCallback(_currentAppObserver, _currentAppElement.axUIElement,
                        kAXFocusedWindowChangedNotification,
                        (__bridge void *)self);

  NSLog(@"Observing process %d for window focus changes", pid);
  return YES;
}

- (void)startObservingWithCallback:(WindowChangeCallback)callback {
  if (_isObserving) {
    [self stopObserving];
  }

  _callback = callback;
  _isObserving = YES;

  // First observe the currently focused application
  NSRunningApplication *currentApp =
      [[NSWorkspace sharedWorkspace] frontmostApplication];
  if (currentApp) {
    [self startObservingApp:currentApp.processIdentifier];
  }

  NSLog(@"Starting app switch observer");
  // Register for workspace notifications to detect app switching
  __weak typeof(self) weakSelf = self;

  // Make sure we're on the main thread when setting up the notification
  // observer
  // Remove any existing observer first
  if (self->_appSwitchObserver) {
    [[NSWorkspace sharedWorkspace].notificationCenter
        removeObserver:self->_appSwitchObserver];
    self->_appSwitchObserver = nil;
  }

  // Add the new observer
  self->_appSwitchObserver = [[NSWorkspace sharedWorkspace].notificationCenter
      addObserverForName:NSWorkspaceDidActivateApplicationNotification
                  object:nil
                   queue:[NSOperationQueue mainQueue]
              usingBlock:^(NSNotification *notification) {
                NSRunningApplication *app =
                    notification.userInfo[NSWorkspaceApplicationKey];
                NSLog(@"Focused app changed to: %@ (Bundle ID: %@)",
                      app.localizedName, app.bundleIdentifier);
                [weakSelf startObservingApp:app.processIdentifier];
              }];

  NSLog(@"App switch observer registered: %@", self->_appSwitchObserver);
  NSRunLoop *currentRunLoop = [NSRunLoop currentRunLoop];

  // Add a port to keep the run loop alive (needed for background threads)
  NSPort *port = [NSPort port];
  [currentRunLoop addPort:port forMode:NSDefaultRunLoopMode];

  [currentRunLoop run];
}

- (void)stopObserving {
  if (!_isObserving) {
    return;
  }

  // Remove app switch observer
  if (_appSwitchObserver) {
    [[NSWorkspace sharedWorkspace].notificationCenter
        removeObserver:_appSwitchObserver];
    _appSwitchObserver = nil;
  }

  // Clean up app observer
  [self cleanupCurrentAppObserver];

  // Clean up window observers
  for (NSValue *key in _windowObservers) {
    AXObserverRef observer =
        (__bridge AXObserverRef)([_windowObservers objectForKey:key]);
    CFRunLoopRemoveSource(CFRunLoopGetCurrent(),
                          AXObserverGetRunLoopSource(observer),
                          kCFRunLoopDefaultMode);
  }
  [_windowObservers removeAllObjects];

  _callback = NULL;
  _isObserving = NO;
}

- (void)cleanupCurrentAppObserver {
  if (_currentAppElement && _currentAppObserver) {
    [_currentAppElement removeObserver:_currentAppObserver
                          notification:kAXFocusedWindowChangedNotification];
    _currentAppElement = nil;

    CFRunLoopRemoveSource(CFRunLoopGetCurrent(),
                          AXObserverGetRunLoopSource(_currentAppObserver),
                          kCFRunLoopDefaultMode);
    CFRelease(_currentAppObserver);
    _currentAppObserver = NULL;
  }
}

- (BOOL)isObserving {
  return _isObserving;
}

void free_window_title(WindowTitle *window_title);

@end