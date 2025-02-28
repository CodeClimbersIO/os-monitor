#import "WindowUtils.h"
#import "AccessibilityUtils.h"
#import <ApplicationServices/ApplicationServices.h>

void printAttributes(AXUIElementRef element, int depth, int maxDepth) {
  if (!element)
    return;
  if (depth > maxDepth)
    return;
  CFArrayRef attributeNames;
  AXUIElementCopyAttributeNames(element, &attributeNames);
  NSArray *attributes = (__bridge_transfer NSArray *)attributeNames;

  CFStringRef titleRef;
  AXUIElementCopyAttributeValue(element, kAXTitleAttribute,
                                (CFTypeRef *)&titleRef);
  NSString *title = (__bridge_transfer NSString *)titleRef;

  CFStringRef roleRef;
  AXUIElementCopyAttributeValue(element, kAXRoleAttribute,
                                (CFTypeRef *)&roleRef);
  NSString *role = (__bridge_transfer NSString *)roleRef;

  // Create indent based on depth with colors
  char indent[100] = "";
  // ANSI foreground color codes from 31-36 (red, green, cyan, blue, magenta,
  // yellow)
  int colorCode = 31 + (depth % 6);
  for (int i = 0; i < depth; i++) {
    strcat(indent, "  "); // Just add spaces without color
  }

  // Add color code at the start of the line, but after the indent
  char colorStart[20];
  sprintf(colorStart, "\033[%dm", colorCode);

  // Reset color code at the end of indent
  char resetColor[] = "\033[0m";

  printf("\n%s%s=== Element at depth %d ===%s\n", indent, colorStart, depth,
         resetColor);
  printf("%s%sRole: %s%s\n", indent, colorStart, [role UTF8String], resetColor);
  if (title) {
    printf("%s%sTitle: %s%s\n", indent, colorStart, [title UTF8String],
           resetColor);
  }

  for (NSString *attribute in attributes) {
    CFTypeRef valueRef;
    AXError error = AXUIElementCopyAttributeValue(
        element, (__bridge CFStringRef)attribute, &valueRef);

    if (error == kAXErrorSuccess) {
      id value = (__bridge_transfer id)valueRef;
      printf("%s%sAttribute: %s = %s%s\n", indent, colorStart,
             [attribute UTF8String], [[value description] UTF8String],
             resetColor);

      // Recursively explore children
      if ([attribute isEqualToString:NSAccessibilityChildrenAttribute]) {
        NSArray *children = (NSArray *)value;
        for (id child in children) {
          printAttributes((__bridge AXUIElementRef)child, depth + 1, maxDepth);
        }
      }
    } else {
      printf("%s%sAttribute: %s (Error getting value: %d)%s\n", indent,
             colorStart, [attribute UTF8String], error, resetColor);
    }
  }
  printf("%s%s===========================%s\n\n", indent, colorStart,
         resetColor);
}

BOOL isDomain(NSString *str) {
  NSString *pattern = @"^(?:https?:\\/\\/"
                      @")?(?:www\\.)?[a-zA-Z0-9-]+(?:\\.[a-zA-Z0-9-]+)*\\.[a-"
                      @"zA-Z]{2,}(?:\\/[^\\s]*)?(?:\\?[^\\s]*)?$";
  NSError *error = nil;
  NSRegularExpression *regex =
      [NSRegularExpression regularExpressionWithPattern:pattern
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
 * Find the URL element in the given accessibility element. Recursively searches
 * through children. Assumes that the URL element is a static text or a text
 * field.
 * TODO: future versions might need to search specifically based on the browser
 * @param element The accessibility element to search
 * @return The URL element if found, otherwise NULL
 */
AXUIElementRef findUrlElement(AXUIElementRef element) {
  if (!element)
    return NULL;

  CFStringRef roleRef;
  AXUIElementCopyAttributeValue(element, kAXRoleAttribute,
                                (CFTypeRef *)&roleRef);
  NSString *role = (__bridge_transfer NSString *)roleRef;

  if ([role isEqualToString:NSAccessibilityStaticTextRole] ||
      [role isEqualToString:NSAccessibilityTextFieldRole]) {
    CFTypeRef valueRef;
    AXError error =
        AXUIElementCopyAttributeValue(element, kAXValueAttribute, &valueRef);
    if (error == kAXErrorSuccess) {
      NSString *value = (__bridge_transfer NSString *)valueRef;
      if (isDomain(value)) {
        CFRetain(element);
        return element;
      }
    }
  }

  CFArrayRef childrenRef;
  AXError childrenError = AXUIElementCopyAttributeValue(
      element, kAXChildrenAttribute, (CFTypeRef *)&childrenRef);

  if (childrenError == kAXErrorSuccess) {
    NSArray *children = (__bridge_transfer NSArray *)childrenRef;
    for (id child in children) {
      AXUIElementRef urlElement =
          findUrlElement((__bridge AXUIElementRef)child);
      if (urlElement != NULL) {
        return urlElement;
      }
    }
  }

  return NULL;
}

BOOL isSupportedBrowser(NSString *bundleId) {
  return [bundleId isEqualToString:@"com.apple.Safari"] ||
         [bundleId isEqualToString:@"com.google.Chrome"] ||
         [bundleId isEqualToString:@"com.microsoft.Edge"] ||
         [bundleId isEqualToString:@"com.google.Chrome.canary"] ||
         [bundleId isEqualToString:@"com.google.Chrome.beta"] ||
         [bundleId isEqualToString:@"company.thebrowser.Browser"];
}

NSRunningApplication *get_frontmost_app(void) {
  // Get all windows
  CFArrayRef windowList = CGWindowListCopyWindowInfo(
      kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
      kCGNullWindowID);

  NSArray *windows = (__bridge_transfer NSArray *)windowList;
  NSDictionary *frontWindow = nil;

  for (NSDictionary *window in windows) {
    NSNumber *layer = window[(id)kCGWindowLayer];
    NSNumber *alpha = window[(id)kCGWindowAlpha];

    if ([layer intValue] == 0 && [alpha floatValue] > 0) {
      frontWindow = window;
      break;
    }
  }

  if (!frontWindow) {
    NSLog(@"No front window found");
    return nil;
  }

  // Get the process ID of the frontmost window
  pid_t pid = [frontWindow[(id)kCGWindowOwnerPID] intValue];
  NSRunningApplication *frontmostApp =
      [NSRunningApplication runningApplicationWithProcessIdentifier:pid];

  if (!frontmostApp) {
    NSLog(@"Failed to get frontmost application");
    return nil;
  }
  return frontmostApp;
}

WindowTitle *detect_focused_window(void) {
  NSRunningApplication *frontmostApp = get_frontmost_app();

  if (!frontmostApp) {
    NSLog(@"Failed to get frontmost application");
    return nil;
  }

  NSString *bundleId = frontmostApp.bundleIdentifier;

  AXUIElementRef appRef =
      AXUIElementCreateApplication(frontmostApp.processIdentifier);
  if (!appRef) {
    NSLog(@"Failed to create accessibility element for application");
    return nil;
  }
  AXUIElementRef focusedWindow;
  AXError result = AXUIElementCopyAttributeValue(
      appRef, kAXFocusedWindowAttribute, (CFTypeRef *)&focusedWindow);
  if (result != kAXErrorSuccess) {
    NSLog(@"Debug - Raw error code: %d, kAXErrorCannotComplete: %d",
          (int)result, (int)kAXErrorCannotComplete);
    NSLog(@"Failed to get focused window: %@ (error code: %d)",
          getAXErrorDescription(result), result);
    CFRelease(appRef);
    return nil;
  }

  CFTypeRef windowTitle;
  result = AXUIElementCopyAttributeValue(focusedWindow, kAXTitleAttribute,
                                         &windowTitle);
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
    WindowTitle *windowTitleStruct = malloc(sizeof(WindowTitle));

    windowTitleStruct->window_title = strdup([title UTF8String]);
    windowTitleStruct->app_name = strdup([applicationName UTF8String]);
    windowTitleStruct->bundle_id =
        bundleId ? strdup([bundleId UTF8String]) : NULL;
    windowTitleStruct->url = url ? strdup([url UTF8String]) : NULL;

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