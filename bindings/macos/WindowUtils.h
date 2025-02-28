#import <Cocoa/Cocoa.h>

typedef struct {
  const char *app_name;
  const char *window_title;
  const char *bundle_id;
  const char *url;
} WindowTitle;

WindowTitle *detect_focused_window(void);

BOOL isDomain(NSString *str);

BOOL isSupportedBrowser(NSString *bundleId);

NSRunningApplication *get_frontmost_app(void);

AXUIElementRef findUrlElement(AXUIElementRef element);

void printAttributes(AXUIElementRef element, int depth, int maxDepth);