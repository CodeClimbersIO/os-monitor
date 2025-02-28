#import "SiteBlocker.h"
#import "Application.h"
#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>

// Static variables to store state
static BOOL siteBlockingEnabled = NO;
static NSMutableArray<NSString *> *blockedUrls = nil;
static NSString *vibesUrl = nil;

// Function declarations (prototypes)
void simulateKeyPress(CGKeyCode keyCode, CGEventFlags flags);
AXUIElementRef findURLFieldInElement(AXUIElementRef element,
                                     NSString *bundleId);

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

// // Find URL field in a browser window using Accessibility API
// AXUIElementRef findURLFieldInElement(AXUIElementRef element,
//                                      NSString *bundleId) {
//   if (!element)
//     return NULL;
//   // Get the role
//   CFStringRef roleRef;
//   AXUIElementCopyAttributeValue(element, kAXRoleAttribute,
//                                 (CFTypeRef *)&roleRef);
//   NSString *role = (__bridge_transfer NSString *)roleRef;

//   // Check if this is a text field (potential URL field)
//   // NSLog(@"isTextField: %@ %@", role, [role
//   // isEqualToString:NSAccessibilityTextFieldRole]);
//   printAttributes(element, 0, 3);
//   // Use description approach for Chromium-based browsers
//   if (isChromiumBrowser(bundleId)) {
//     if ([role isEqualToString:NSAccessibilityTextFieldRole]) {
//       // Get the description or identifier
//       CFStringRef descriptionRef;
//       if (AXUIElementCopyAttributeValue(element, kAXDescriptionAttribute,
//                                         (CFTypeRef *)&descriptionRef) ==
//           kAXErrorSuccess) {
//         NSString *description = (__bridge_transfer NSString *)descriptionRef;

//         // Check if this is a URL field based on its description
//         NSArray *urlIdentifiers = @[
//           @"Address", @"URL", @"Location", @"Address and search bar",
//           @"address field"
//         ];
//         for (NSString *identifier in urlIdentifiers) {
//           if ([description rangeOfString:identifier
//                                  options:NSCaseInsensitiveSearch]
//                   .location != NSNotFound) {
//             CFRetain(element); // Retain it since we'll pass it to the caller
//             return element;
//           }
//         }
//       }
//     }
//   } else if (isSafari(bundleId)) {
//     // Safari-specific approach
//     // Safari's URL field is typically a text field with specific
//     // characteristics Could check for the element's position or other
//     // attributes

//     // For now using a simplified approach:
//     CFStringRef valueRef;
//     if (AXUIElementCopyAttributeValue(element, kAXValueAttribute,
//                                       (CFTypeRef *)&valueRef) ==
//         kAXErrorSuccess) {
//       NSString *value = (__bridge_transfer NSString *)valueRef;
//       // Check if the value looks like a URL
//       if ([value hasPrefix:@"http://"] || [value hasPrefix:@"https://"] ||
//           [value hasPrefix:@"www."] || [value containsString:@"."]) {
//         CFRetain(element);
//         return element;
//       }
//     }
//   } else if (isArc(bundleId)) {
//     NSLog(@"Arc browser detected");

//     // Check for the identifier attribute
//     CFStringRef identifierRef;
//     if (AXUIElementCopyAttributeValue(element, kAXIdentifierAttribute,
//                                       (CFTypeRef *)&identifierRef) ==
//         kAXErrorSuccess) {
//       NSString *identifier = (__bridge_transfer NSString *)identifierRef;

//       // Check if this is the URL field based on the identifier
//       if ([identifier isEqualToString:@"commandBarPlaceholderTextField"]) {
//         CFRetain(element); // Retain it since we'll pass it to the caller
//         return element;
//       }
//     }
//   }
//   // Add other browser-specific detection logic here as needed

//   // Recursively search children
//   CFArrayRef childrenRef;
//   AXError childrenError = AXUIElementCopyAttributeValue(
//       element, kAXChildrenAttribute, (CFTypeRef *)&childrenRef);

//   if (childrenError == kAXErrorSuccess) {
//     NSArray *children = (__bridge_transfer NSArray *)childrenRef;
//     for (id child in children) {
//       AXUIElementRef urlField =
//           findURLFieldInElement((__bridge AXUIElementRef)child, bundleId);
//       if (urlField) {
//         return urlField;
//       }
//     }
//   }
//   return NULL;
// }

BOOL start_site_blocking(const char **blocked_urls, int url_count,
                         const char *redirect_url) {
  NSLog(@"start_site_blocking");
  @autoreleasepool {
    // Check if redirect_url is provided (now required)
    if (!redirect_url) {
      NSLog(@"Error: redirect_url is required");
      return NO; // Return failure if redirect_url is not provided
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

BOOL is_url_blocked(const char *url) {
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
      if ([currentUrl rangeOfString:blockedUrl options:NSCaseInsensitiveSearch]
              .location != NSNotFound) {
        NSLog(@"URL %@ is blocked (matched %@)", currentUrl, blockedUrl);
        return YES;
      }
    }

    return NO;
  }
}

BOOL redirectUsingAppleScript(NSString *browserBundleId, NSString *targetUrl) {
  NSString *scriptText = nil;
  NSString *browserName = nil;

  if ([browserBundleId isEqualToString:@"com.apple.Safari"]) {
    browserName = @"Safari";
    scriptText = [NSString
        stringWithFormat:
            @"tell application \"%@\" to set URL of front document to \"%@\"",
            browserName, targetUrl];
  } else if ([browserBundleId isEqualToString:@"com.google.Chrome"]) {
    browserName = @"Google Chrome";
    scriptText =
        [NSString stringWithFormat:@"tell application \"%@\" to set URL of "
                                   @"active tab of front window to \"%@\"",
                                   browserName, targetUrl];
  } else if ([browserBundleId isEqualToString:@"org.mozilla.firefox"]) {
    browserName = @"Firefox";
    // Firefox has limited AppleScript support but can open URLs
    scriptText = [NSString
        stringWithFormat:@"tell application \"%@\" to open location \"%@\"",
                         browserName, targetUrl];
  } else if ([browserBundleId isEqualToString:@"com.microsoft.edgemac"]) {
    browserName = @"Microsoft Edge";
    scriptText =
        [NSString stringWithFormat:@"tell application \"%@\" to set URL of "
                                   @"active tab of front window to \"%@\"",
                                   browserName, targetUrl];
  } else if ([browserBundleId isEqualToString:@"company.thebrowser.Browser"]) {
    browserName = @"Arc";
    scriptText =
        [NSString stringWithFormat:@"tell application \"%@\" to set URL of "
                                   @"active tab of front window to \"%@\"",
                                   browserName, targetUrl];
  } else {
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

BOOL hasAutomationPermission(NSString *bundleId) {
  NSAppleScript *testScript = [[NSAppleScript alloc]
      initWithSource:[NSString stringWithFormat:
                                   @"tell application id \"%@\" to return name",
                                   bundleId]];
  NSDictionary *errorInfo = nil;
  [testScript executeAndReturnError:&errorInfo];

  return (errorInfo == nil);
}

BOOL redirect_to_vibes_page(void) {
  @autoreleasepool {
    NSLog(@"Redirecting to vibes page");

    // Use FocusedApp to get the frontmost application
    FocusedApp *frontApp = [FocusedApp frontmostApp];
    if (!frontApp) {
      NSLog(@"Failed to get frontmost application");
      return NO;
    }

    if (![frontApp isSupportedBrowser]) {
      NSLog(@"Unsupported browser: %@", frontApp.bundleId);
      return NO;
    }

    // Important: Move the redirection to a background queue to avoid blocking
    dispatch_async(
        dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
          if (hasAutomationPermission(frontApp.bundleId)) {
            if (redirectUsingAppleScript(frontApp.bundleId, vibesUrl)) {
              NSLog(@"Successfully redirected using AppleScript");
              return;
            }
          }

          NSLog(@"AppleScript redirection failed, falling back to "
                @"accessibility API");

          // Get the focused window using FocusedApp
          AppWindow *window = [frontApp focusedWindow];
          if (!window) {
            NSLog(@"Failed to get browser window");
            return;
          }

          AccessibilityElement *urlField = [window findAddressBar];

          if (urlField) {
            NSLog(@"Found URL field, setting focus");
            AXUIElementSetAttributeValue(urlField.axUIElement,
                                         kAXFocusedAttribute, kCFBooleanTrue);
            usleep(100000); // 100ms delay

            // Select all text (Cmd+A)
            simulateKeyPress(0,
                             kCGEventFlagMaskCommand); // 'A' key with Command
            usleep(50000);                             // 50ms delay

            // Set the URL
            NSLog(@"Setting URL to: %@", vibesUrl);
            AXError axError = AXUIElementSetAttributeValue(
                urlField.axUIElement, kAXValueAttribute,
                (__bridge CFTypeRef)vibesUrl);

            if (axError == kAXErrorSuccess) {
              NSLog(@"Successfully set URL, pressing Enter");
              usleep(100000); // 100ms delay
              // Press Enter to navigate
              simulateKeyPress(36, 0); // Return key
            }
          }
        });

    return YES;
  }
}

BOOL request_automation_permission(const char *bundle_id) {
  if (!bundle_id)
    return NO;

  NSString *bundleIdStr = [NSString stringWithUTF8String:bundle_id];

  NSString *scriptText =
      [NSString stringWithFormat:@"tell application id \"%@\" to return name",
                                 bundleIdStr];
  NSAppleScript *script = [[NSAppleScript alloc] initWithSource:scriptText];

  NSDictionary *errorInfo = nil;
  [script executeAndReturnError:&errorInfo];

  return hasAutomationPermission(bundleIdStr);
}