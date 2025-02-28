#import "Application.h"
#import "AccessibilityElement.h"
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

BOOL isSupportedBrowser(NSString *bundleId) {
  return [bundleId isEqualToString:@"com.apple.Safari"] ||
         [bundleId isEqualToString:@"com.google.Chrome"] ||
         [bundleId isEqualToString:@"com.microsoft.Edge"] ||
         [bundleId isEqualToString:@"com.google.Chrome.canary"] ||
         [bundleId isEqualToString:@"com.google.Chrome.beta"] ||
         [bundleId isEqualToString:@"company.thebrowser.Browser"];
}

@implementation AppWindow

- (instancetype)initWithAccessibilityElement:(AccessibilityElement *)element {
  self = [super init];
  if (self) {
    _accessibilityElement = element;
  }
  return self;
}

- (instancetype)initWithAXUIElement:(AXUIElementRef)element {
  AccessibilityElement *accessibilityElement =
      [[AccessibilityElement alloc] initWithAXUIElement:element];
  return [self initWithAccessibilityElement:accessibilityElement];
}

- (void)dealloc {
  // AccessibilityElement will handle releasing the AXUIElement
}

- (NSString *)title {
  return [_accessibilityElement title];
}

- (NSString *)url {
  AccessibilityElement *urlElement = [_accessibilityElement findUrlElement];
  if (urlElement) {
    return [urlElement value];
  }
  return nil;
}

- (AXUIElementRef)axUIElement {
  return _accessibilityElement.axUIElement;
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

- (void)dealloc {
  // AccessibilityElement will handle releasing the AXUIElement
}

- (NSString *)appName {
  return _runningApplication.localizedName;
}

- (NSString *)bundleId {
  return _runningApplication.bundleIdentifier;
}

- (AppWindow *)focusedWindow {
  AccessibilityElement *focusedWindowElement =
      [_accessibilityElement valueForAttribute:kAXFocusedWindowAttribute];

  if (!focusedWindowElement) {
    NSLog(@"Failed to get focused window");
    return nil;
  }

  AppWindow *window =
      [[AppWindow alloc] initWithAccessibilityElement:focusedWindowElement];
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