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
BOOL isSupportedBrowser(NSString *bundleId);

@interface AppWindow : NSObject
@property(nonatomic) AXUIElementRef axUIElement;
- (instancetype)initWithAXUIElement:(AXUIElementRef)element;
- (NSString *)title;
- (NSString *)url;
@end

@interface FocusedApp : NSObject
@property(nonatomic) AXUIElementRef axUIElement;
@property(nonatomic, strong) NSRunningApplication *runningApplication;
+ (instancetype)frontmostApp;
+ (NSRunningApplication *)getFrontmostApp;
- (instancetype)initWithRunningApplication:(NSRunningApplication *)app;
- (NSString *)appName;
- (NSString *)bundleId;
- (AppWindow *)focusedWindow;
- (WindowTitle *)windowTitleStructWithWindow;
@end
