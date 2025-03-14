#import "Application.h"
#import "AccessibilityElement.h"
#import "AccessibilityUtils.h"
#import <ApplicationServices/ApplicationServices.h>

BOOL isDomain(NSString *str) {
  if (![str isKindOfClass:[NSString class]]) {
    return NO;
  }

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

@implementation AppWindow

- (instancetype)initWithAccessibilityElement:(AccessibilityElement *)element
                                   parentApp:(FocusedApp *)parentApp {
  self = [super init];
  if (self) {
    _accessibilityElement = element;
    _parentApp = parentApp;
  }
  return self;
}

- (NSString *)title {
  return [_accessibilityElement title];
}

- (NSString *)url {
  if (![_parentApp isSupportedBrowser]) {
    return nil;
  }
  AccessibilityElement *urlElement = [self findUrlElement];
  if (urlElement) {
    NSString *rawUrl = [urlElement value];
    NSLog(@"rawUrl: %@", rawUrl);
    return rawUrl;
  }
  return nil;
}

/**
 * Find the URL element in the given accessibility element. Recursively searches
 * through children. Assumes that the URL element is a static text or a text
 * field.
 * @return The URL element if found, otherwise nil
 */
- (AccessibilityElement *)findUrlElement {
  return [self findUrlElementInElement:_accessibilityElement depth:0];
}

/**
 * Helper method to recursively search for URL element
 * @param element The accessibility element to search
 * @param depth Current recursion depth
 * @return The URL element if found, otherwise nil
 */
- (AccessibilityElement *)findUrlElementInElement:
                              (AccessibilityElement *)element
                                            depth:(int)depth {
  if (!element || depth >= 10)
    return nil;

  NSString *role = [element role];
  if ([role isEqualToString:NSAccessibilityStaticTextRole] ||
      [role isEqualToString:NSAccessibilityTextFieldRole]) {
    NSString *value = [element value];
    if (value && isDomain(value)) {
      return element;
    }
  }

  NSArray *children = [element children];
  for (id child in children) {
    AccessibilityElement *childElement = [[AccessibilityElement alloc]
        initWithAXUIElement:(__bridge AXUIElementRef)child];
    AccessibilityElement *urlElement =
        [self findUrlElementInElement:childElement depth:depth + 1];
    if (urlElement != nil) {
      return urlElement;
    }
  }

  return nil;
}

/**
 * Find the URL field in the given accessibility element. Recursively searches
 * through children. Meant to be used to find the address bar in a browser.
 * @return The URL field if found, otherwise nil
 */
- (AccessibilityElement *)findAddressBar {
  return [self findAddressBarInElement:_accessibilityElement depth:0];
}

/**
 * Helper method to recursively search for address bar
 * @param element The accessibility element to search
 * @param depth Current recursion depth
 * @return The address bar element if found, otherwise nil
 */
- (AccessibilityElement *)findAddressBarInElement:
                              (AccessibilityElement *)element
                                            depth:(int)depth {
  if (!element.axUIElement || depth >= 10)
    return nil;

  // Get the role
  NSString *role = [element role];

  // Use description approach for Chromium-based browsers
  if ([_parentApp isChromiumBrowser]) {
    if ([role isEqualToString:NSAccessibilityTextFieldRole]) {
      // Get the description
      NSString *description = [element description];

      if (description) {
        // Check if this is a URL field based on its description
        NSArray *urlIdentifiers = @[
          @"Address", @"URL", @"Location", @"Address and search bar",
          @"address field"
        ];
        for (NSString *identifier in urlIdentifiers) {
          if ([description rangeOfString:identifier
                                 options:NSCaseInsensitiveSearch]
                  .location != NSNotFound) {
            return element;
          }
        }
      }
    }
    // Safari-specific approach
  } else if ([_parentApp isSafari]) {
    id value = [element value];

    // Check if the value is a string and looks like a URL
    if ([value isKindOfClass:[NSString class]] &&
        ([(NSString *)value hasPrefix:@"http://"] ||
         [(NSString *)value hasPrefix:@"https://"] ||
         [(NSString *)value hasPrefix:@"www."] ||
         [(NSString *)value containsString:@"."])) {
      return element;
    }
  } else if ([_parentApp isArc]) {

    // Check for the identifier attribute
    NSString *identifier = [element identifier];

    // Check if this is the URL field based on the identifier
    if ([identifier isEqualToString:@"commandBarPlaceholderTextField"]) {
      return element;
    }
  }

  // Recursively search children
  NSArray *children = [element children];
  for (id child in children) {
    AccessibilityElement *childElement = [[AccessibilityElement alloc]
        initWithAXUIElement:(__bridge AXUIElementRef)child];
    AccessibilityElement *urlField = [self findAddressBarInElement:childElement
                                                             depth:depth + 1];
    if (urlField) {
      return urlField;
    }
  }

  return nil;
}

- (AccessibilityElement *)accessibilityElement {
  return _accessibilityElement;
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
    AXUIElementRef axElement =
        AXUIElementCreateApplication(app.processIdentifier);
    _accessibilityElement =
        [[AccessibilityElement alloc] initWithAXUIElement:axElement];
    CFRelease(axElement); // AccessibilityElement retains it
  }
  return self;
}

- (NSString *)appName {
  return _runningApplication.localizedName;
}

- (NSString *)bundleId {
  return _runningApplication.bundleIdentifier;
}

- (NSString *)url {
  if (![self isSupportedBrowser]) {
    return nil;
  }

  AppWindow *window = [self focusedWindow];
  if (!window) {
    NSLog(@"Failed to get focused window");
    return nil;
  }
  return [window url];
}

- (AppWindow *)focusedWindow {
  AccessibilityElement *focusedWindowElement =
      [_accessibilityElement valueForAttribute:kAXFocusedWindowAttribute];

  if (!focusedWindowElement) {
    NSLog(@"Failed to get focused window");
    return nil;
  }

  AppWindow *window =
      [[AppWindow alloc] initWithAccessibilityElement:focusedWindowElement
                                            parentApp:self];
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

- (AXUIElementRef)axUIElement {
  return _accessibilityElement.axUIElement;
}

- (BOOL)isSupportedBrowser {
  return [self isSafari] || [self isChromiumBrowser] || [self isArc];
}

- (BOOL)isChromiumBrowser {
  NSArray *chromiumBrowsers = @[
    @"com.google.Chrome",
    @"com.google.Chrome.beta",
    @"com.google.Chrome.dev",
    @"com.google.Chrome.canary",
  ];

  return [chromiumBrowsers containsObject:self.bundleId];
}

- (BOOL)isSafari {
  return [self.bundleId isEqualToString:@"com.apple.Safari"];
}

- (BOOL)isArc {
  return [self.bundleId isEqualToString:@"company.thebrowser.Browser"];
}

@end

// Modified version of detect_focused_window to use our new classes
WindowTitle *detect_focused_window(void) {
  @try {
    FocusedApp *app = [FocusedApp frontmostApp];
    if (!app) {
      NSLog(@"Failed to get frontmost application");
      return nil;
    }
    return [app windowTitleStructWithWindow];
  } @catch (NSException *exception) {
    NSLog(@"Exception: %@", exception);
    @throw exception;
  }
}

// Export a C function to get the frontmost app for use in Blocker.m
NSRunningApplication *get_frontmost_app(void) {
  return [FocusedApp getFrontmostApp];
}

void free_window_title(WindowTitle *window_title) {
  if (window_title) {
    if (window_title->app_name) {
      free((void *)window_title->app_name);
    }
    if (window_title->window_title) {
      free((void *)window_title->window_title);
    }
    if (window_title->bundle_id) {
      free((void *)window_title->bundle_id);
    }
    if (window_title->url) {
      free((void *)window_title->url);
    }
    free(window_title);
  }
}