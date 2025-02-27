#import "SiteBlocker.h"
#import "WindowUtils.h"
#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>

// Static variables to store state
static BOOL siteBlockingEnabled = NO;
static NSMutableArray<NSString*> *blockedUrls = nil;
static NSString *vibesUrl = nil; 

// Function declarations (prototypes)
void simulateKeyPress(CGKeyCode keyCode, CGEventFlags flags);
AXUIElementRef findURLFieldInElement(AXUIElementRef element);

// Simulate a key press
void simulateKeyPress(CGKeyCode keyCode, CGEventFlags flags) {
    CGEventRef keyDown = CGEventCreateKeyboardEvent(NULL, keyCode, true);
    CGEventRef keyUp = CGEventCreateKeyboardEvent(NULL, keyCode, false);
    
    CGEventSetFlags(keyDown, flags);
    CGEventSetFlags(keyUp, flags);
    
    CGEventPost(kCGSessionEventTap, keyDown);
    usleep(10000); // Short delay
    CGEventPost(kCGSessionEventTap, keyUp);
    
    CFRelease(keyDown);
    CFRelease(keyUp);
}

// Find URL field in a browser window using Accessibility API
AXUIElementRef findURLFieldInElement(AXUIElementRef element) {
    if (!element) return NULL;
    
    // Get the role
    CFStringRef roleRef;
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute, (CFTypeRef *)&roleRef);
    NSString *role = (__bridge_transfer NSString *)roleRef;
    
    // Check if this is a text field (potential URL field)
    if ([role isEqualToString:NSAccessibilityTextFieldRole]) {
        // Get the description or identifier
        CFStringRef descriptionRef;
        if (AXUIElementCopyAttributeValue(element, kAXDescriptionAttribute, (CFTypeRef *)&descriptionRef) == kAXErrorSuccess) {
            NSString *description = (__bridge_transfer NSString *)descriptionRef;
            
            // Check if this is a URL field based on its description
            NSArray *urlIdentifiers = @[@"Address", @"URL", @"Location", @"Address and search bar", @"address field"];
            for (NSString *identifier in urlIdentifiers) {
                if ([description rangeOfString:identifier options:NSCaseInsensitiveSearch].location != NSNotFound) {
                    CFRetain(element); // Retain it since we'll pass it to the caller
                    return element;
                }
            }
        }
    }
    
    // Recursively search children
    CFArrayRef childrenRef;
    AXError childrenError = AXUIElementCopyAttributeValue(element, kAXChildrenAttribute, (CFTypeRef *)&childrenRef);
    
    if (childrenError == kAXErrorSuccess) {
        NSArray *children = (__bridge_transfer NSArray *)childrenRef;
        for (id child in children) {
            AXUIElementRef urlField = findURLFieldInElement((__bridge AXUIElementRef)child);
            if (urlField) {
                return urlField;
            }
        }
    }
    
    return NULL;
}

BOOL start_site_blocking(const char** blocked_urls, int url_count, const char* redirect_url) {
    NSLog(@"start_site_blocking");
    @autoreleasepool {
        // Check if redirect_url is provided (now required)
        if (!redirect_url) {
            NSLog(@"Error: redirect_url is required");
            return NO;  // Return failure if redirect_url is not provided
        }
        
        if (blockedUrls == nil) {
            blockedUrls = [NSMutableArray array];
        } else {
            [blockedUrls removeAllObjects];
        }
        
        // Copy the URLs from C strings to NSString objects
        for (int i = 0; i < url_count; i++) {
            if (blocked_urls[i]) {
                NSString *url = [NSString stringWithUTF8String:blocked_urls[i]];
                [blockedUrls addObject:url];
                NSLog(@"Blocking URL: %@", url);
            }
        }
        
        // Set the redirect URL (now required)
        vibesUrl = [NSString stringWithUTF8String:redirect_url];
        NSLog(@"Redirect URL set to: %@", vibesUrl);
        
        siteBlockingEnabled = YES;
        NSLog(@"Site blocking enabled");
        return YES;
    }
}

void stop_site_blocking(void) {
    siteBlockingEnabled = NO;
    NSLog(@"Site blocking disabled");
}

BOOL is_url_blocked(const char* url) {
    NSLog(@"is_url_blocked");
    NSLog(@"url: %s", url);
    if (!url || !siteBlockingEnabled || blockedUrls.count == 0) {
        return NO;
    }
    
    @autoreleasepool {
        NSString *currentUrl = [NSString stringWithUTF8String:url];
        
        // Check if the current URL contains any of the blocked URLs
        for (NSString *blockedUrl in blockedUrls) {
            NSLog(@"blockedUrl: %@", blockedUrl);
            NSLog(@"currentUrl: %@", currentUrl);
            if ([currentUrl rangeOfString:blockedUrl options:NSCaseInsensitiveSearch].location != NSNotFound) {
                NSLog(@"URL %@ is blocked (matched %@)", currentUrl, blockedUrl);
                return YES;
            }
        }
        
        return NO;
    }
}

BOOL redirect_to_vibes_page(void) {
    @autoreleasepool {
        NSLog(@"Redirecting to vibes page");
        NSRunningApplication *frontApp = get_frontmost_app();
        if (!frontApp) {
            NSLog(@"Failed to get frontmost application");
            return NO;
        }
        
        NSString *bundleId = frontApp.bundleIdentifier;
        NSLog(@"Browser bundle ID: %@", bundleId);
        
        // Check if this is a supported browser
        if (!isSupportedBrowser(bundleId)) {
            NSLog(@"Unsupported browser: %@", bundleId);
            return NO;
        }
        
        // Create AXUIElement for the application
        AXUIElementRef appElement = AXUIElementCreateApplication(frontApp.processIdentifier);
        if (!appElement) {
            NSLog(@"Failed to create AX element for app");
            return NO;
        }
        
        // Find the window
        AXUIElementRef window = NULL;
        AXError axError = AXUIElementCopyAttributeValue(appElement, kAXFocusedWindowAttribute, (CFTypeRef *)&window);
        
        if (axError != kAXErrorSuccess || !window) {
            // Try to get the first window
            CFArrayRef windowArray = NULL;
            axError = AXUIElementCopyAttributeValues(appElement, kAXWindowsAttribute, 0, 1, &windowArray);
            
            if (axError == kAXErrorSuccess && windowArray) {
                if (CFArrayGetCount(windowArray) > 0) {
                    window = (AXUIElementRef)CFRetain(CFArrayGetValueAtIndex(windowArray, 0));
                }
                CFRelease(windowArray);
            }
        }
        
        if (!window) {
            NSLog(@"Could not find browser window (AX error: %d)", axError);
            CFRelease(appElement);
            return NO;
        }
        
        // Dispatch the rest of the work to avoid blocking
        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            // Find the URL field
            AXUIElementRef urlField = findURLFieldInElement(window);
            __block AXError blockAxError;
            
            if (urlField) {
                NSLog(@"Found URL field, setting focus");
                // Focus the URL field
                AXUIElementSetAttributeValue(urlField, kAXFocusedAttribute, kCFBooleanTrue);
                usleep(100000); // 100ms delay
                
                // Select all text (Cmd+A)
                simulateKeyPress(0, kCGEventFlagMaskCommand); // 'A' key with Command
                usleep(50000); // 50ms delay
                
                // Set the URL
                NSLog(@"Setting URL to: %@", vibesUrl);
                blockAxError = AXUIElementSetAttributeValue(urlField, kAXValueAttribute, (__bridge CFTypeRef)vibesUrl);
                
                if (blockAxError == kAXErrorSuccess) {
                    NSLog(@"Successfully set URL, pressing Enter");
                    usleep(100000); // 100ms delay
                    // Press Enter to navigate
                    simulateKeyPress(36, 0); // Return key
                } else {
                    NSLog(@"Failed to set URL (AX error: %d)", blockAxError);
                    
                    // Fallback: try key sequence instead
                    // Press Cmd+L to focus address bar
                    simulateKeyPress(37, kCGEventFlagMaskCommand); // Cmd+L
                    usleep(100000); // 100ms delay
                    
                    // Type the URL (not implemented here, would need character-by-character simulation)
                    // For simplicity, we'll just use the clipboard
                    NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
                    [pasteboard clearContents];
                    [pasteboard writeObjects:@[vibesUrl]];
                    usleep(50000); // 50ms delay
                    
                    // Press Cmd+V to paste
                    simulateKeyPress(9, kCGEventFlagMaskCommand); // Cmd+V
                    usleep(100000); // 100ms delay
                    
                    // Press Enter
                    simulateKeyPress(36, 0); // Return key
                }
                
                CFRelease(urlField);
            } else {
                NSLog(@"Could not find URL field, trying fallback approach");
                
                // Fallback: try key sequence
                // Press Cmd+L to focus address bar
                simulateKeyPress(37, kCGEventFlagMaskCommand); // Cmd+L
                usleep(100000); // 100ms delay
                
                // Use clipboard to set URL
                NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
                [pasteboard clearContents];
                [pasteboard writeObjects:@[vibesUrl]];
                usleep(50000); // 50ms delay
                
                // Press Cmd+V to paste
                simulateKeyPress(9, kCGEventFlagMaskCommand); // Cmd+V
                usleep(100000); // 100ms delay
                
                // Press Enter
                simulateKeyPress(36, 0); // Return key
            }
            
            CFRelease(window);
            CFRelease(appElement);
        });
        
        return YES;
    }
} 