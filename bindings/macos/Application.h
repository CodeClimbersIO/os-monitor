#import <Cocoa/Cocoa.h>

typedef struct {
  const char *app_name;
  const char *window_title;
  const char *bundle_id;
  const char *url;
} WindowTitle;

WindowTitle *detect_focused_window(void);
void free_window_title(WindowTitle *window_title);
AXUIElementRef findUrlElement(AXUIElementRef element);
void printAttributes(AXUIElementRef element, int depth, int maxDepth);
BOOL isDomain(NSString *str);

@class AccessibilityElement;
@class FocusedApp;

@interface AppWindow : NSObject
@property(nonatomic, strong) AccessibilityElement *accessibilityElement;
@property(nonatomic, weak) FocusedApp *parentApp;
- (instancetype)initWithAccessibilityElement:(AccessibilityElement *)element
                                   parentApp:(FocusedApp *)parentApp;
- (NSString *)title;
- (NSString *)url;

- (AccessibilityElement *)findUrlElement;
- (AccessibilityElement *)findUrlElementInElement:
    (AccessibilityElement *)element;
- (AccessibilityElement *)findAddressBar;
- (AccessibilityElement *)findAddressBarInElement:
    (AccessibilityElement *)element;
@end

@interface FocusedApp : NSObject
@property(nonatomic) AXUIElementRef axUIElement;
@property(nonatomic, strong) NSRunningApplication *runningApplication;
@property(nonatomic, strong) AccessibilityElement *accessibilityElement;
+ (instancetype)frontmostApp;
+ (NSRunningApplication *)getFrontmostApp;
- (instancetype)initWithRunningApplication:(NSRunningApplication *)app;
- (NSString *)appName;
- (NSString *)bundleId;
- (AppWindow *)focusedWindow;
- (WindowTitle *)windowTitleStructWithWindow;
- (BOOL)isSupportedBrowser;
- (BOOL)isChromiumBrowser;
- (BOOL)isSafari;
- (BOOL)isArc;
@end
