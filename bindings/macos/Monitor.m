// Monitor.m

#import "Monitor.h"
#import <objc/runtime.h>
#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>

@interface MonitorHolder : NSObject
@property (nonatomic, strong) NSArray<id> *monitors;
@property (nonatomic, assign) MouseEventCallback mouseCallback;
@property (nonatomic, assign) KeyboardEventCallback keyboardCallback;
@property (nonatomic, assign) WindowEventCallback windowCallback;
@end

@implementation MonitorHolder
@end

static MonitorHolder *monitorHolder = nil;

// All window detection functions have been moved to WindowUtils.m:
// - printAttributes()
// - isDomain()
// - findUrlElement()
// - isSupportedBrowser()
// - get_frontmost_app()
// - detect_focused_window()

CGEventRef eventCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *refcon) {
    CGPoint location = CGEventGetLocation(event);
    switch (type) {
        case kCGEventKeyDown:
            if (monitorHolder.keyboardCallback) {
                CGKeyCode keyCode = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
                monitorHolder.keyboardCallback((int32_t)keyCode);
                
                // Check if it's an Enter key (keyCode 36) and handle site blocking
                if (keyCode == 36) {
                    handle_enter_key_for_site_blocking();
                }
            }
            break;
            
        case kCGEventLeftMouseDown:
            if (monitorHolder.mouseCallback) {
                monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeLeftDown, 0);
            }
            break;
            
        case kCGEventLeftMouseUp:
            if (monitorHolder.mouseCallback) {
                monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeLeftUp, 0);
            }
            break;
            
        case kCGEventRightMouseDown:
            if (monitorHolder.mouseCallback) {
                monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeRightDown, 0);
            }
            break;
            
        case kCGEventRightMouseUp:
            if (monitorHolder.mouseCallback) {
                monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeRightUp, 0);
            }
            break;
            
        case kCGEventMouseMoved:
            if (monitorHolder.mouseCallback) {
                monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeMove, 0);
            }
            break;
            
        case kCGEventScrollWheel:
            if (monitorHolder.mouseCallback) {
                int32_t scrollDelta = (int32_t)CGEventGetIntegerValueField(event, kCGScrollWheelEventDeltaAxis1);
                monitorHolder.mouseCallback(location.x, location.y, MouseEventTypeScroll, scrollDelta);
            }
            break;
    }
    
    return event;
}

void start_monitoring(MouseEventCallback mouseCallback, KeyboardEventCallback keyboardCallback) {
    if (!has_accessibility_permissions()) {
        NSLog(@"start_monitoring - No accessibility permissions");
        return;
    }
    if (!monitorHolder) {
        monitorHolder = [[MonitorHolder alloc] init];
    }
    monitorHolder.mouseCallback = mouseCallback;
    monitorHolder.keyboardCallback = keyboardCallback;
    // Create event tap for mouse clicks, movements, and key events
    CGEventMask eventMask = CGEventMaskBit(kCGEventLeftMouseDown) | 
                           CGEventMaskBit(kCGEventLeftMouseUp) |
                           CGEventMaskBit(kCGEventRightMouseDown) |
                           CGEventMaskBit(kCGEventRightMouseUp) |
                           CGEventMaskBit(kCGEventMouseMoved) |
                           CGEventMaskBit(kCGEventScrollWheel) |
                           CGEventMaskBit(kCGEventKeyDown) |
                           CGEventMaskBit(kCGEventKeyUp);
    CFMachPortRef _eventTap = CGEventTapCreate(kCGSessionEventTap,
                                kCGHeadInsertEventTap,
                                kCGEventTapOptionDefault,
                                eventMask,
                                eventCallback,
                                NULL);
    if (!_eventTap) {
        NSLog(@"Failed to create event tap");
        return;
    }
    CFRunLoopSourceRef _runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, _eventTap, 0);
    CFRunLoopAddSource(CFRunLoopGetCurrent(), _runLoopSource, kCFRunLoopCommonModes);
    CGEventTapEnable(_eventTap, true);

    [[NSRunLoop currentRunLoop] run];
}

const char* get_app_icon_data(const char* bundle_id) {
    if (!bundle_id) return NULL;
    
    @autoreleasepool {
        NSString *bundleIdStr = [NSString stringWithUTF8String:bundle_id];
        NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
        
        // // Get the icon for the application
        NSImage *icon = [workspace iconForFile:[[workspace URLForApplicationWithBundleIdentifier:bundleIdStr] path]];
        if (!icon) return NULL;
        
        // // Convert to PNG data
        NSBitmapImageRep *imageRep = [NSBitmapImageRep imageRepWithData:[icon TIFFRepresentation]];
        NSData *pngData = [imageRep representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        if (!pngData) return NULL;
        
        // // Convert to base64
        NSString *base64String = [pngData base64EncodedStringWithOptions:0];
        NSString *dataUrl = [NSString stringWithFormat:@"data:image/png;base64,%@", base64String];
        
        // Copy the string to a new buffer that will be freed by the caller
        const char *utf8String = [dataUrl UTF8String];
        char *result = strdup(utf8String);
        return result;
    }
}

void free_icon_data(const char* data) {
    if (data) {
        free((void*)data);
    }
}