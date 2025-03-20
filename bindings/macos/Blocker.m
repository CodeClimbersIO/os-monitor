#import "Blocker.h"

// Static variables to store state
static BOOL siteBlockingEnabled = NO;
static NSMutableArray<NSString *> *blockedApps = nil;
static NSString *vibesUrl = nil;
static AppBlockedCallback appBlockedCallback = NULL;

// Arrays to store blocked app information
static NSMutableArray<NSString *> *batchAppNames = nil;
static NSMutableArray<NSString *> *batchBundleIds = nil;

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

void sendBlockedAppsBatch() {
  if (appBlockedCallback != NULL && batchAppNames.count > 0) {
    const char **appNamesCArray =
        (const char **)malloc(batchAppNames.count * sizeof(char *));
    const char **bundleIdsCArray =
        (const char **)malloc(batchBundleIds.count * sizeof(char *));

    for (NSUInteger i = 0; i < batchAppNames.count; i++) {
      appNamesCArray[i] = [batchAppNames[i] UTF8String];
      bundleIdsCArray[i] = [batchBundleIds[i] UTF8String];
    }
    appBlockedCallback(appNamesCArray, bundleIdsCArray,
                       (int)batchAppNames.count);

    free(appNamesCArray);
    free(bundleIdsCArray);

    [batchAppNames removeAllObjects];
    [batchBundleIds removeAllObjects];
  }
}

void addAppToBlockedBatch(NSString *appName, NSString *bundleId) {
  if (batchAppNames == nil) {
    batchAppNames = [NSMutableArray array];
  }

  if (batchBundleIds == nil) {
    batchBundleIds = [NSMutableArray array];
  }

  [batchAppNames addObject:appName];
  [batchBundleIds addObject:bundleId];
}

void closeBlockedApplications(void) {
  @autoreleasepool {
    for (NSString *blockedAppId in blockedApps) {
      close_app([blockedAppId UTF8String], NO);
    }

    sendBlockedAppsBatch();
  }
}

BOOL start_blocking(const char **blocked_urls, int url_count,
                    const char *redirect_url) {
  NSLog(@"start_blocking");
  @autoreleasepool {
    if (!redirect_url) {
      NSLog(@"Error: redirect_url is required");
      return NO;
    }

    if (blockedApps == nil) {
      blockedApps = [NSMutableArray array];
    } else {
      [blockedApps removeAllObjects];
    }

    for (int i = 0; i < url_count; i++) {
      if (blocked_urls[i]) {
        NSString *url = [NSString stringWithUTF8String:blocked_urls[i]];
        [blockedApps addObject:url];
        NSLog(@"Blocking URL: %@", url);
      }
    }

    vibesUrl = [NSString stringWithUTF8String:redirect_url];
    NSLog(@"Redirect URL set to: %@", vibesUrl);
    siteBlockingEnabled = YES;
    closeBlockedApplications();
    NSLog(@"Site blocking enabled");
    return YES;
  }
}

void stop_blocking(void) {
  siteBlockingEnabled = NO;
  NSLog(@"Site blocking disabled");
}

BOOL close_app(const char *external_app_id, const bool send_callback) {
  if (!external_app_id || !siteBlockingEnabled || blockedApps.count == 0) {
    return NO;
  }

  @autoreleasepool {
    NSString *currentAppId = [NSString stringWithUTF8String:external_app_id];
    NSArray *runningApps = [NSRunningApplication
        runningApplicationsWithBundleIdentifier:currentAppId];
    NSRunningApplication *blockedApp = [runningApps firstObject];
    if (blockedApp) {
      NSLog(@"Terminating blocked application: %@", blockedApp);
      [blockedApp terminate];
      NSLog(@"Successfully terminated blocked application: %@", blockedApp);
      addAppToBlockedBatch([blockedApp localizedName],
                           [blockedApp bundleIdentifier]);
      if (send_callback) {
        sendBlockedAppsBatch();
      }
      return YES;
    }
  }
  return NO;
}

BOOL is_blocked(const char *external_app_id) {
  if (!external_app_id || !siteBlockingEnabled || blockedApps.count == 0) {
    return NO;
  }

  FocusedApp *frontApp = [FocusedApp frontmostApp];
  BOOL isUrlElementFocused = [frontApp isUrlElementFocused];
  if (isUrlElementFocused) {
    return NO;
  }

  @autoreleasepool {
    NSString *currentAppId = [NSString stringWithUTF8String:external_app_id];
    NSString *cleanedAppId = [currentAppId componentsSeparatedByString:@"/"][0];
    NSLog(@"cleanedAppId: %@", cleanedAppId);
    for (NSString *blockedAppId in blockedApps) {
      if ([cleanedAppId caseInsensitiveCompare:blockedAppId] == NSOrderedSame) {
        NSLog(@"App ID %@ is blocked (exact match with %@)", currentAppId,
              blockedAppId);

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
  } else if ([browserBundleId isEqualToString:@"com.brave.Browser"]) {
    browserName = @"Brave Browser";
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

  // Create a temporary AppleScript file with the script
  NSString *tempDir = NSTemporaryDirectory();
  NSString *scriptPath =
      [tempDir stringByAppendingPathComponent:@"browser_redirect.scpt"];

  NSError *error = nil;
  BOOL success = [scriptText writeToFile:scriptPath
                              atomically:YES
                                encoding:NSUTF8StringEncoding
                                   error:&error];

  if (!success) {
    NSLog(@"Failed to write script file: %@", error);
    return NO;
  }

  // Execute the script using osascript directly
  NSTask *task = [[NSTask alloc] init];
  [task setLaunchPath:@"/usr/bin/osascript"];
  [task setArguments:@[ scriptPath ]];

  NSPipe *pipe = [NSPipe pipe];
  [task setStandardOutput:pipe];
  [task setStandardError:pipe];
  [task launch];

  // Wait for a reasonable amount of time
  [task waitUntilExit];
  int status = [task terminationStatus];

  // Clean up
  [[NSFileManager defaultManager] removeItemAtPath:scriptPath error:nil];

  if (status != 0) {
    NSLog(@"osascript exited with status: %d", status);
    NSFileHandle *fileHandle = [pipe fileHandleForReading];
    NSData *data = [fileHandle readDataToEndOfFile];
    NSString *output = [[NSString alloc] initWithData:data
                                             encoding:NSUTF8StringEncoding];
    NSLog(@"osascript output: %@", output);
    return NO;
  }

  NSLog(@"osascript executed successfully");
  return YES;
}

BOOL requestAutomationPermission(const char *bundle_id) {
  if (!bundle_id)
    return NO;

  NSString *bundleIdStr = [NSString stringWithUTF8String:bundle_id];

  NSString *scriptText =
      [NSString stringWithFormat:@"tell application id \"%@\" to return name",
                                 bundleIdStr];
  NSAppleScript *script = [[NSAppleScript alloc] initWithSource:scriptText];

  NSDictionary *errorInfo = nil;
  [script executeAndReturnError:&errorInfo];

  NSAppleEventDescriptor *result = [script executeAndReturnError:&errorInfo];

  if (errorInfo) {
    NSLog(@"AppleScript error: %@", errorInfo);
    return NO;
  }
  return YES;
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

    requestAutomationPermission([frontApp.bundleId UTF8String]);

    BOOL redirectResult = redirectUsingAppleScript(frontApp.bundleId, vibesUrl);
    if (!redirectResult) {
      NSLog(@"Failed to redirect using AppleScript for %@", frontApp.bundleId);
      commandBarRedirect();
    }

    NSLog(@"Successfully redirected to vibes page");
    return YES; // Return success for the main function since we've started
                // the async process
  }
}

void register_app_blocked_callback(AppBlockedCallback callback) {
  appBlockedCallback = callback;

  if (batchAppNames == nil) {
    batchAppNames = [NSMutableArray array];
  }

  if (batchBundleIds == nil) {
    batchBundleIds = [NSMutableArray array];
  }
}
