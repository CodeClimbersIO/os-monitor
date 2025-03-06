#import "SiteBlocker.h"
#import "Application.h"
#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>

// Static variables to store state
static BOOL siteBlockingEnabled = NO;
static NSMutableArray<NSString *> *blockedUrls = nil;
static NSString *vibesUrl = nil;

void simulateKeyPress(CGKeyCode keyCode, CGEventFlags flags);

void fallbackNavigation(NSString *url);

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
  if (!url || !siteBlockingEnabled || blockedUrls.count == 0) {
    return NO;
  }

  @autoreleasepool {
    NSString *currentUrl = [NSString stringWithUTF8String:url];

    // Check if the current URL contains any of the blocked URLs
    for (NSString *blockedUrl in blockedUrls) {
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

  NSLog(@"browserBundleId: %@", browserBundleId);
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

void commandBarRedirect() {

  NSLog(@"using command bar redirect");
  // Fallback: try key sequence
  // Press Cmd+L to focus address bar
  simulateKeyPress(37, kCGEventFlagMaskCommand); // Cmd+L
  usleep(100000);                                // 100ms delay

  // Use clipboard to set URL
  NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
  [pasteboard clearContents];
  [pasteboard writeObjects:@[ vibesUrl ]];
  usleep(50000); // 50ms delay

  // Press Cmd+V to paste
  simulateKeyPress(9, kCGEventFlagMaskCommand); // Cmd+V
  usleep(100000);                               // 100ms delay

  // Press Enter
  simulateKeyPress(36, 0); // Return key
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

    commandBarRedirect();

    NSLog(@"Successfully redirected to vibes page");
    return YES; // Return success for the main function since we've started the
                // async process
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