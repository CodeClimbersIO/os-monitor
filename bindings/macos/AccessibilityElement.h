#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>

@interface AccessibilityElement : NSObject

@property(nonatomic, readonly) AXUIElementRef axUIElement;

- (instancetype)initWithAXUIElement:(AXUIElementRef)element;
- (void)focus;
- (void)printAttributesWithDepth:(int)depth maxDepth:(int)maxDepth;
- (void)printAttributes;

// Attribute accessors
- (NSString *)role;
- (NSString *)description;
- (NSString *)identifier;
- (NSString *)value;
- (NSString *)title;
- (NSArray *)children;

// Get a specific attribute as an AccessibilityElement
- (AccessibilityElement *)valueForAttribute:(CFStringRef)attribute;
- (AXError)setValue:(NSString *)value;

// New observer methods
- (BOOL)addObserver:(AXObserverRef)observer
       notification:(CFStringRef)notification
           callback:(AXObserverCallback)callback
           userData:(void *)userData;
- (void)removeObserver:(AXObserverRef)observer
          notification:(CFStringRef)notification;

@end