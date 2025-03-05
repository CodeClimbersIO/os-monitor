#import "AccessibilityElement.h"
#import "Application.h"
#import <Cocoa/Cocoa.h>

// Callback function type definition for window changes
typedef void (*WindowChangeCallback)(const char *app_name,
                                     const char *window_title,
                                     const char *bundle_id, const char *url);

@interface WindowObserver : NSObject

+ (instancetype)sharedObserver;

// Start/stop observing window changes
- (void)startObservingWithCallback:(WindowChangeCallback)callback;
- (void)stopObserving;

// Check if we're currently observing
- (BOOL)isObserving;

@end