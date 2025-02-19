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
static NSString* getAXErrorDescription(AXError error) {
    switch (error) {
        case kAXErrorCannotComplete:           
            return @"Cannot complete the operation (window might be transitioning)";
        case kAXErrorNotImplemented:           
            return @"The accessibility feature is not implemented";
        case kAXErrorInvalidUIElement:         
            return @"Invalid UI element";
        case kAXErrorFailure:                  
            return @"Operation failed";
        case kAXErrorIllegalArgument:          
            return @"Illegal argument";
        case kAXErrorNoValue:                 
            return @"No value available";
        case kAXErrorAPIDisabled:              
            return @"Accessibility API disabled";
        case kAXErrorNotificationUnsupported:  
            return @"Notification not supported";
        default:
            return [NSString stringWithFormat:@"Unknown error code: %d", (int)error];
    }
}


BOOL isDomain(NSString *str) {
    NSString *pattern = @"^(?:https?:\\/\\/)?(?:www\\.)?[a-zA-Z0-9-]+(?:\\.[a-zA-Z0-9-]+)*\\.[a-zA-Z]{2,}(?:\\/[^\\s]*)?(?:\\?[^\\s]*)?$";
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern
                                                                         options:0
                                                                           error:&error];
    if (error) {
        return NO;
    }
    
    NSRange range = NSMakeRange(0, [str length]);
    NSArray *matches = [regex matchesInString:str options:0 range:range];
    
    return matches.count > 0;
}

/**
* Find the URL element in the given accessibility element. Recursively searches through children.
* Assumes that the URL element is a static text or a text field. 
* TODO: future versions might need to search specifically based on the browser
* @param element The accessibility element to search
* @return The URL element if found, otherwise NULL
*/
AXUIElementRef findUrlElement(AXUIElementRef element) {
    if (!element) return NULL;
    
    CFStringRef roleRef;
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute, (CFTypeRef *)&roleRef);
    NSString *role = (__bridge_transfer NSString *)roleRef;
    
    if ([role isEqualToString:NSAccessibilityStaticTextRole] || [role isEqualToString:NSAccessibilityTextFieldRole]) {
        CFTypeRef valueRef;
        AXError error = AXUIElementCopyAttributeValue(element, kAXValueAttribute, &valueRef);
        if (error == kAXErrorSuccess) {
            NSString *value = (__bridge_transfer NSString *)valueRef;
            if (isDomain(value)) {
                CFRetain(element);
                return element;
            }
        }
    }
    
    CFArrayRef childrenRef;
    AXError childrenError = AXUIElementCopyAttributeValue(element, kAXChildrenAttribute, (CFTypeRef *)&childrenRef);
    
    if (childrenError == kAXErrorSuccess) {
        NSArray *children = (__bridge_transfer NSArray *)childrenRef;
        for (id child in children) {
            AXUIElementRef urlElement = findUrlElement((__bridge AXUIElementRef)child);
            if (urlElement != NULL) {
                return urlElement;
            }
        }
    }
    
    return NULL;
}

BOOL has_accessibility_permissions(void) {
    NSDictionary *options = @{(__bridge id)kAXTrustedCheckOptionPrompt: @NO};
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

BOOL request_accessibility_permissions(void) {
    NSDictionary *options = @{(__bridge id)kAXTrustedCheckOptionPrompt: @YES};
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

BOOL isSupportedBrowser(NSString *bundleId) {
    // return true;
    return [bundleId isEqualToString:@"com.apple.Safari"] ||
           [bundleId isEqualToString:@"com.google.Chrome"] ||
           [bundleId isEqualToString:@"com.microsoft.Edge"] ||
           [bundleId isEqualToString:@"com.google.Chrome.canary"] ||
           [bundleId isEqualToString:@"com.google.Chrome.beta"] ||
           [bundleId isEqualToString:@"company.thebrowser.Browser"];
}

WindowTitle* detect_focused_window(void) {
    if (!has_accessibility_permissions()) {
        return nil;
    }

        
    // Create system-wide accessibility element
    // Get the frontmost application process first
    NSRunningApplication *frontmostApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    
    if (!frontmostApp) {
        NSLog(@"Failed to get frontmost application");
        return nil;
    }
    NSString *bundleId = frontmostApp.bundleIdentifier;
    
    // Create an accessibility element for the specific application
    AXUIElementRef appRef = AXUIElementCreateApplication(frontmostApp.processIdentifier);
    if (!appRef) {
        NSLog(@"Failed to create accessibility element for application");
        return nil;
    }
    // Get focused window
    AXUIElementRef focusedWindow;
    AXError result = AXUIElementCopyAttributeValue(
        appRef,
        kAXFocusedWindowAttribute,
        (CFTypeRef *)&focusedWindow
    );
    if (result != kAXErrorSuccess) {
        NSLog(@"Debug - Raw error code: %d, kAXErrorCannotComplete: %d", 
            (int)result, (int)kAXErrorCannotComplete);
        NSLog(@"Failed to get focused window: %@ (error code: %d)", 
            getAXErrorDescription(result), result);
        CFRelease(appRef);
        return nil;
    }
    
    // Get window title
    CFTypeRef windowTitle;
    result = AXUIElementCopyAttributeValue(
        focusedWindow,
        kAXTitleAttribute,
        &windowTitle
    );
    // Get window URL
    NSString *url = nil;
    if (isSupportedBrowser(bundleId)) {
        AXUIElementRef urlElement = findUrlElement(focusedWindow);
        if (urlElement) {
            CFTypeRef valueRef;
            AXUIElementCopyAttributeValue(urlElement, kAXValueAttribute, &valueRef);
            url = (__bridge_transfer NSString *)valueRef;
        }
    }
    
    if (result == kAXErrorSuccess) {
        NSString *title = (__bridge_transfer NSString *)windowTitle;
        NSString *applicationName = frontmostApp.localizedName; 
        WindowTitle* windowTitleStruct = malloc(sizeof(WindowTitle));
        
        // Create copies of all strings
        windowTitleStruct->window_title = strdup([title UTF8String]);
        windowTitleStruct->app_name = strdup([applicationName UTF8String]);
        windowTitleStruct->bundle_id = bundleId ? strdup([bundleId UTF8String]) : NULL;
        windowTitleStruct->url = url ? strdup([url UTF8String]) : NULL;

        // Clean up AX resources
        CFRelease(focusedWindow);
        CFRelease(appRef);
        
        return windowTitleStruct;
    } else {
        NSLog(@"Debug - Raw error code: %d, kAXErrorCannotComplete: %d", 
            (int)result, (int)kAXErrorCannotComplete);
        NSLog(@"Failed to get window title: %@ (error code: %d)", 
            getAXErrorDescription(result), result);
        CFRelease(appRef);
        return nil;
    }
    
    // Clean up
    CFRelease(focusedWindow);
    CFRelease(appRef);
    return nil;
}

CGEventRef eventCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *refcon) {
    CGPoint location = CGEventGetLocation(event);
    switch (type) {
        case kCGEventKeyDown:
            if (monitorHolder.keyboardCallback) {
                CGKeyCode keyCode = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
                monitorHolder.keyboardCallback((int32_t)keyCode);
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
    printf("Event mask: %d\n", eventMask);
    CFMachPortRef _eventTap = CGEventTapCreate(kCGSessionEventTap,
                                kCGHeadInsertEventTap,
                                kCGEventTapOptionDefault,
                                eventMask,
                                eventCallback,
                                NULL);
    printf("Event tap: %p\n", _eventTap);
    if (!_eventTap) {
        NSLog(@"Failed to create event tap");
        return;
    }
    printf("Event tap created\n");
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
        
        // Get the icon for the application
        NSImage *icon = [workspace iconForFile:[[workspace URLForApplicationWithBundleIdentifier:bundleIdStr] path]];
        if (!icon) return NULL;
        
        // Convert to PNG data
        NSBitmapImageRep *imageRep = [NSBitmapImageRep imageRepWithData:[icon TIFFRepresentation]];
        NSData *pngData = [imageRep representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        if (!pngData) return NULL;
        
        // Convert to base64
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