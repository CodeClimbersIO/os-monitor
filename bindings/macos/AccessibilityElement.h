#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>

@interface AccessibilityElement : NSObject

@property(readonly) AXUIElementRef axUIElement;

- (instancetype)initWithAXUIElement:(AXUIElementRef)element;
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

- (AccessibilityElement *)findUrlElement;
- (AccessibilityElement *)findAddressBarForBrowser:(NSString *)bundleId;

@end

BOOL isChromiumBrowser(NSString *bundleId);
BOOL isSafari(NSString *bundleId);
BOOL isArc(NSString *bundleId);
BOOL isDomain(NSString *str);

void printAttributes(AXUIElementRef element, int indent, int maxDepth);
