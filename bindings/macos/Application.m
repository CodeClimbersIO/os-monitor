#import "Application.h"
#import "AccessibilityUtils.h"
#import <ApplicationServices/ApplicationServices.h>

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

@implementation AppWindow

- (instancetype)initWithAXUIElement:(AXUIElementRef)element {
  self = [super init];
  if (self) {
    _axUIElement = element;
    CFRetain(_axUIElement);
  }
  return self;
}

- (void)dealloc {
  if (_axUIElement) {
    CFRelease(_axUIElement);
  }
}

- (NSString *)title {
  CFTypeRef windowTitle;
  AXError result = AXUIElementCopyAttributeValue(
      _axUIElement, kAXTitleAttribute, &windowTitle);

  if (result == kAXErrorSuccess) {
    NSString *title = (__bridge_transfer NSString *)windowTitle;
    return title;
  }

  return nil;
}

- (NSString *)url {
  AXUIElementRef urlElement = findUrlElement(_axUIElement);
  if (urlElement) {
    CFTypeRef valueRef;
    AXUIElementCopyAttributeValue(urlElement, kAXValueAttribute, &valueRef);
    NSString *url = (__bridge_transfer NSString *)valueRef;
    CFRelease(urlElement);
    return url;
  }
  return nil;
}

@end

@implementation FocusedApp

+ (instancetype)frontmostApp {
  NSRunningApplication *frontmostApp = [self getFrontmostApp];
  if (!frontmostApp) {
    return nil;
  }

  return [[FocusedApp alloc] initWithRunningApplication:frontmostApp];
}

+ (NSRunningApplication *)getFrontmostApp {
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

- (instancetype)initWithRunningApplication:(NSRunningApplication *)app {
  self = [super init];
  if (self) {
    _runningApplication = app;
    _axUIElement = AXUIElementCreateApplication(app.processIdentifier);
  }
  return self;
}

- (void)dealloc {
  if (_axUIElement) {
    CFRelease(_axUIElement);
  }
}

- (NSString *)appName {
  return _runningApplication.localizedName;
}

- (NSString *)bundleId {
  return _runningApplication.bundleIdentifier;
}

- (AppWindow *)focusedWindow {
  AXUIElementRef focusedWindow;
  AXError result = AXUIElementCopyAttributeValue(
      _axUIElement, kAXFocusedWindowAttribute, (CFTypeRef *)&focusedWindow);

  if (result != kAXErrorSuccess) {
    NSLog(@"Failed to get focused window: %@ (error code: %d)",
          getAXErrorDescription(result), result);
    return nil;
  }

  AppWindow *window = [[AppWindow alloc] initWithAXUIElement:focusedWindow];
  CFRelease(focusedWindow);
  return window;
}

- (WindowTitle *)windowTitleStructWithWindow {
  AppWindow *window = [self focusedWindow];
  if (!window) {
    NSLog(@"Failed to get focused window");
    return nil;
  }

  NSString *title = [window title];
  NSString *url = [window url];
  NSString *appName = [self appName];
  NSString *bundleId = [self bundleId];

  if (title) {
    WindowTitle *windowTitleStruct = malloc(sizeof(WindowTitle));

    windowTitleStruct->window_title = strdup([title UTF8String]);
    windowTitleStruct->app_name = strdup([appName UTF8String]);
    windowTitleStruct->bundle_id =
        bundleId ? strdup([bundleId UTF8String]) : NULL;
    windowTitleStruct->url = url ? strdup([url UTF8String]) : NULL;

    return windowTitleStruct;
  }

  return NULL;
}

@end

// Modified version of detect_focused_window to use our new classes
WindowTitle *detect_focused_window(void) {
  FocusedApp *app = [FocusedApp frontmostApp];
  if (!app) {
    NSLog(@"Failed to get frontmost application");
    return nil;
  }
  return [app windowTitleStructWithWindow];
}

// Export a C function to get the frontmost app for use in SiteBlocker.m
NSRunningApplication *get_frontmost_app(void) {
  return [FocusedApp getFrontmostApp];
}