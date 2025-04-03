#import "Monitor.h"
#import "UI.h"
#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>
#import <objc/runtime.h>

typedef void (*WebsiteVisitCallback)(const char *url);

@interface MonitorHolder : NSObject
@property(nonatomic, strong) NSArray<id> *monitors;
@property(nonatomic, assign) MouseEventCallback mouseCallback;
@property(nonatomic, assign) KeyboardEventCallback keyboardCallback;
@property(nonatomic, assign) WindowEventCallback windowCallback;
@property(nonatomic, assign) WebsiteVisitCallback websiteCallback;
@property(nonatomic, strong) id axObserver;
@end

@implementation MonitorHolder
@end

static MonitorHolder *monitorHolder = nil;

void run_loop_cycle() {
  NSLog(@"Running run loop for 10ms");
  NSDate *stopDate =
      [NSDate dateWithTimeIntervalSinceNow:0.01]; // 1/100th of a second
  [[NSRunLoop currentRunLoop] runUntilDate:stopDate];
}

void start_run_loop() {
  @autoreleasepool {
    NSLog(@"Processing events");
    NSLog(@"Thread: %@", [NSThread currentThread]);

    NSRunLoop *currentRunLoop = [NSRunLoop currentRunLoop];

    [currentRunLoop run];
    NSLog(@"Processing events end");
  }
}

CGEventRef eventCallback(CGEventTapProxy proxy, CGEventType type,
                         CGEventRef event, void *refcon) {
  CGPoint location = CGEventGetLocation(event);
  switch (type) {
  case kCGEventKeyDown:
    if (monitorHolder.keyboardCallback) {
      int64_t keyCode =
          CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);

      monitorHolder.keyboardCallback((int32_t)keyCode);
    }
    break;

  case kCGEventLeftMouseDown:
    if (monitorHolder.mouseCallback) {
      monitorHolder.mouseCallback(location.x, location.y,
                                  MouseEventTypeLeftDown, 0);
    }
    break;

  case kCGEventLeftMouseUp:
    if (monitorHolder.mouseCallback) {
      monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeLeftUp,
                                  0);
    }
    break;

  case kCGEventRightMouseDown:
    if (monitorHolder.mouseCallback) {
      monitorHolder.mouseCallback(location.x, location.y,
                                  MouseEventTypeRightDown, 0);
    }
    break;

  case kCGEventRightMouseUp:
    if (monitorHolder.mouseCallback) {
      monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeRightUp,
                                  0);
    }
    break;

  case kCGEventMouseMoved:
    if (monitorHolder.mouseCallback) {
      monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeMove,
                                  0);
    }
    break;

  case kCGEventScrollWheel:
    if (monitorHolder.mouseCallback) {
      int32_t scrollDelta = (int32_t)CGEventGetIntegerValueField(
          event, kCGScrollWheelEventDeltaAxis1);
      monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeScroll,
                                  scrollDelta);
    }
    break;

  default:
    // Handle any other event types or just do nothing
    break;
  }

  return event;
}

void start_monitoring(MouseEventCallback mouseCallback,
                      KeyboardEventCallback keyboardCallback) {
  if (!has_accessibility_permissions()) {
    NSLog(@"start_monitoring - No accessibility permissions");
    return;
  }
  NSLog(@"start_monitoring");
  if (!monitorHolder) {
    monitorHolder = [[MonitorHolder alloc] init];
  }
  monitorHolder.mouseCallback = mouseCallback;
  monitorHolder.keyboardCallback = keyboardCallback;
  // Create event tap for mouse clicks, movements, and key events
  CGEventMask eventMask =
      CGEventMaskBit(kCGEventLeftMouseDown) |
      CGEventMaskBit(kCGEventLeftMouseUp) |
      CGEventMaskBit(kCGEventRightMouseDown) |
      CGEventMaskBit(kCGEventRightMouseUp) |
      CGEventMaskBit(kCGEventMouseMoved) | CGEventMaskBit(kCGEventScrollWheel) |
      CGEventMaskBit(kCGEventKeyDown) | CGEventMaskBit(kCGEventKeyUp);
  CFMachPortRef _eventTap = CGEventTapCreate(
      kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionDefault,
      eventMask, eventCallback, NULL);
  if (!_eventTap) {
    NSLog(@"Failed to create event tap");
    return;
  }
  CFRunLoopSourceRef _runLoopSource =
      CFMachPortCreateRunLoopSource(kCFAllocatorDefault, _eventTap, 0);
  CFRunLoopAddSource(CFRunLoopGetCurrent(), _runLoopSource,
                     kCFRunLoopCommonModes);
  CGEventTapEnable(_eventTap, true);

  start_run_loop();
}

const char *get_app_icon_data(const char *bundle_id) {
  if (!bundle_id)
    return NULL;

  @autoreleasepool {
    NSString *bundleIdStr = [NSString stringWithUTF8String:bundle_id];
    NSWorkspace *workspace = [NSWorkspace sharedWorkspace];

    // // Get the icon for the application
    NSImage *icon = [workspace
        iconForFile:[[workspace
                        URLForApplicationWithBundleIdentifier:bundleIdStr]
                        path]];
    if (!icon)
      return NULL;

    // // Convert to PNG data
    NSBitmapImageRep *imageRep =
        [NSBitmapImageRep imageRepWithData:[icon TIFFRepresentation]];
    NSData *pngData = [imageRep representationUsingType:NSBitmapImageFileTypePNG
                                             properties:@{}];
    if (!pngData)
      return NULL;

    // // Convert to base64
    NSString *base64String = [pngData base64EncodedStringWithOptions:0];
    NSString *dataUrl =
        [NSString stringWithFormat:@"data:image/png;base64,%@", base64String];

    // Copy the string to a new buffer that will be freed by the caller
    const char *utf8String = [dataUrl UTF8String];
    char *result = strdup(utf8String);
    return result;
  }
}

void free_icon_data(const char *data) {
  if (data) {
    free((void *)data);
  }
}

void create_typewriter_window(double opacity) {
  create_typewriter_window(opacity);
}

void sync_typewriter_window_order() { sync_typewriter_window_order(); }
