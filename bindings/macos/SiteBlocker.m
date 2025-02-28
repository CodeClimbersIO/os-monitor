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
AXUIElementRef findURLFieldInElement(AXUIElementRef element, NSString *bundleId);
BOOL isChromiumBrowser(NSString *bundleId);
BOOL isSafari(NSString *bundleId);
BOOL isArc(NSString *bundleId);
void fallbackNavigation(NSString *url);

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

// Check if a browser is Chromium-based
BOOL isChromiumBrowser(NSString *bundleId) {
    NSArray *chromiumBrowsers = @[
        @"com.google.Chrome", 
        @"com.google.Chrome.beta",
        @"com.google.Chrome.dev",
        @"com.google.Chrome.canary",
    ];
    
    return [chromiumBrowsers containsObject:bundleId];
}

BOOL isSafari(NSString *bundleId) {
    return [bundleId isEqualToString:@"com.apple.Safari"];
}

BOOL isArc(NSString *bundleId) {
    return [bundleId isEqualToString:@"company.thebrowser.Browser"];
}

// Find URL field in a browser window using Accessibility API
AXUIElementRef findURLFieldInElement(AXUIElementRef element, NSString *bundleId) {
    if (!element) return NULL;
    // Get the role
    CFStringRef roleRef;
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute, (CFTypeRef *)&roleRef);
    NSString *role = (__bridge_transfer NSString *)roleRef;
    
    // Check if this is a text field (potential URL field)
    // NSLog(@"isTextField: %@ %@", role, [role isEqualToString:NSAccessibilityTextFieldRole]);
    printAttributes(element, 0, 3);
    // Use description approach for Chromium-based browsers
    if (isChromiumBrowser(bundleId)) {
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
    } else if (isSafari(bundleId)) {
        // Safari-specific approach
        // Safari's URL field is typically a text field with specific characteristics
        // Could check for the element's position or other attributes
        
        // For now using a simplified approach:
        CFStringRef valueRef;
        if (AXUIElementCopyAttributeValue(element, kAXValueAttribute, (CFTypeRef *)&valueRef) == kAXErrorSuccess) {
            NSString *value = (__bridge_transfer NSString *)valueRef;
            // Check if the value looks like a URL
            if ([value hasPrefix:@"http://"] || [value hasPrefix:@"https://"] || 
                [value hasPrefix:@"www."] || [value containsString:@"."]) {
                CFRetain(element);
                return element;
            }
        }
    } else if (isArc(bundleId)) {
        NSLog(@"Arc browser detected");
        
        // Check for the identifier attribute
        CFStringRef identifierRef;
        if (AXUIElementCopyAttributeValue(element, kAXIdentifierAttribute, (CFTypeRef *)&identifierRef) == kAXErrorSuccess) {
            NSString *identifier = (__bridge_transfer NSString *)identifierRef;
            
            // Check if this is the URL field based on the identifier
            if ([identifier isEqualToString:@"commandBarPlaceholderTextField"]) {
                CFRetain(element); // Retain it since we'll pass it to the caller
                return element;
            }
        }
    }
    // Add other browser-specific detection logic here as needed
    
    // Recursively search children
    CFArrayRef childrenRef;
    AXError childrenError = AXUIElementCopyAttributeValue(element, kAXChildrenAttribute, (CFTypeRef *)&childrenRef);
    
    if (childrenError == kAXErrorSuccess) {
        NSArray *children = (__bridge_transfer NSArray *)childrenRef;
        for (id child in children) {
            AXUIElementRef urlField = findURLFieldInElement((__bridge AXUIElementRef)child, bundleId);
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

// Add this function to redirect using AppleScript
BOOL redirectUsingAppleScript(NSString *browserBundleId, NSString *targetUrl) {
    NSString *scriptText = nil;
    NSString *browserName = nil;
    
    // Determine which browser is active and set the appropriate script
    if ([browserBundleId isEqualToString:@"com.apple.Safari"]) {
        browserName = @"Safari";
        scriptText = [NSString stringWithFormat:@"tell application \"%@\" to set URL of front document to \"%@\"", 
                      browserName, targetUrl];
    }
    else if ([browserBundleId isEqualToString:@"com.google.Chrome"]) {
        browserName = @"Google Chrome";
        scriptText = [NSString stringWithFormat:@"tell application \"%@\" to set URL of active tab of front window to \"%@\"", 
                      browserName, targetUrl];
    }
    else if ([browserBundleId isEqualToString:@"org.mozilla.firefox"]) {
        browserName = @"Firefox";
        // Firefox has limited AppleScript support but can open URLs
        scriptText = [NSString stringWithFormat:@"tell application \"%@\" to open location \"%@\"", 
                      browserName, targetUrl];
    }
    else if ([browserBundleId isEqualToString:@"com.microsoft.edgemac"]) {
        browserName = @"Microsoft Edge";
        scriptText = [NSString stringWithFormat:@"tell application \"%@\" to set URL of active tab of front window to \"%@\"", 
                      browserName, targetUrl];
    }
    else if ([browserBundleId isEqualToString:@"company.thebrowser.Browser"]) {
        browserName = @"Arc";
        scriptText = [NSString stringWithFormat:@"tell application \"%@\" to set URL of active tab of front window to \"%@\"", 
                      browserName, targetUrl];
    }
    else {
        return NO; // Unsupported browser
    }
    
    if (!scriptText || !browserName) {
        return NO;
    }

    NSLog(@"Running AppleScript for %@: %@", browserName, scriptText);
    
    NSAppleScript *script = [[NSAppleScript alloc] initWithSource:scriptText];
    NSDictionary *errorInfo = nil;
    NSAppleEventDescriptor *result = [script executeAndReturnError:&errorInfo];
    
    if (errorInfo) {
        NSLog(@"AppleScript error: %@", errorInfo);
        return NO;
    }
    
    NSLog(@"AppleScript executed successfully: %@", result);
    return YES;
}

// Check if the app has automation permissions
BOOL hasAutomationPermission(NSString *bundleId) {
    NSAppleScript *testScript = [[NSAppleScript alloc] initWithSource:
                                [NSString stringWithFormat:@"tell application id \"%@\" to return name", bundleId]];
    NSDictionary *errorInfo = nil;
    [testScript executeAndReturnError:&errorInfo];
    
    // If there's no error, we have permission
    return (errorInfo == nil);
}

// Function to modify the existing redirect_to_vibes_page
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
        
        // Important: Move the redirection to a background queue to avoid blocking
        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            // Try AppleScript redirection first (if we have automation permission)
            if (hasAutomationPermission(bundleId)) {
                if (redirectUsingAppleScript(bundleId, vibesUrl)) {
                    NSLog(@"Successfully redirected using AppleScript");
                    return;
                }
            }
            
            // Fall back to accessibility API if AppleScript fails
            NSLog(@"AppleScript redirection failed, falling back to accessibility API");
            
            // Create AXUIElement for the application
            AXUIElementRef appElement = AXUIElementCreateApplication(frontApp.processIdentifier);
            if (!appElement) {
                NSLog(@"Failed to create AX element for app");
                return;
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
                return;
            }
            
            // Find the URL field
            AXUIElementRef urlField = findURLFieldInElement(window, bundleId);
            
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
                axError = AXUIElementSetAttributeValue(urlField, kAXValueAttribute, (__bridge CFTypeRef)vibesUrl);
                
                if (axError == kAXErrorSuccess) {
                    NSLog(@"Successfully set URL, pressing Enter");
                    usleep(100000); // 100ms delay
                    // Press Enter to navigate
                    simulateKeyPress(36, 0); // Return key
                } else {
                    // Fallback approach
                    NSLog(@"Failed to set URL (AX error: %d), using fallback", axError);
                    fallbackNavigation(vibesUrl);
                }
                
                CFRelease(urlField);
            } else {
                NSLog(@"Could not find URL field, trying fallback approach");
                fallbackNavigation(vibesUrl);
            }
            
            CFRelease(window);
            CFRelease(appElement);
        });
        
        return YES;
    }
}

// Add this helper method to clean up the code
void fallbackNavigation(NSString *url) {
    // Press Cmd+L to focus address bar
    simulateKeyPress(37, kCGEventFlagMaskCommand); // Cmd+L
    usleep(100000); // 100ms delay
    
    // Use clipboard to set URL
    NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
    [pasteboard clearContents];
    [pasteboard writeObjects:@[url]];
    usleep(50000); // 50ms delay
    
    // Press Cmd+V to paste
    simulateKeyPress(9, kCGEventFlagMaskCommand); // Cmd+V
    usleep(100000); // 100ms delay
    
    // Press Enter
    simulateKeyPress(36, 0); // Return key
}

// Function to request automation permission
BOOL request_automation_permission(const char* bundle_id) {
    if (!bundle_id) return NO;
    
    NSString *bundleIdStr = [NSString stringWithUTF8String:bundle_id];
    
    // Create a simple AppleScript that will trigger the permission prompt
    NSString *scriptText = [NSString stringWithFormat:@"tell application id \"%@\" to return name", bundleIdStr];
    NSAppleScript *script = [[NSAppleScript alloc] initWithSource:scriptText];
    
    NSDictionary *errorInfo = nil;
    [script executeAndReturnError:&errorInfo];
    
    // Check if we now have permission
    return hasAutomationPermission(bundleIdStr);
} 