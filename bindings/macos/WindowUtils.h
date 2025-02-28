#import <Cocoa/Cocoa.h>

// Define WindowTitle struct to match the existing struct in Monitor.h
typedef struct {
    const char* app_name;
    const char* window_title;
    const char* bundle_id;
    const char* url;
} WindowTitle;

// Main function for detecting the focused window (called from Rust)
WindowTitle* detect_focused_window(void);

// Helper function to check if URL is valid
BOOL isDomain(NSString *str);

// Helper function to check if the app is a supported browser
BOOL isSupportedBrowser(NSString *bundleId);

// Helper function to get frontmost application
NSRunningApplication* get_frontmost_app(void);

// Helper function to find URL element in accessibility hierarchy
AXUIElementRef findUrlElement(AXUIElementRef element);

// Debug utility to print accessibility attributes
void printAttributes(AXUIElementRef element, int depth, int maxDepth); 