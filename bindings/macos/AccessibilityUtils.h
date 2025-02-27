#import <Cocoa/Cocoa.h>

// Helper function to get human-readable error descriptions
NSString* getAXErrorDescription(AXError error);

// Check if the app has accessibility permissions
BOOL has_accessibility_permissions(void);

// Request accessibility permissions from the user
BOOL request_accessibility_permissions(void); 