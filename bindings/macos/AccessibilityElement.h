#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>

@interface AccessibilityElement : NSObject

@property(nonatomic) AXUIElementRef axUIElement;
- (instancetype)initWithAXUIElement:(AXUIElementRef)element;
- (AXUIElementRef)findURLFieldInElementForBrowser:(NSString *)bundleId;
- (void)printAttributes;
- (void)printAttributesWithDepth:(int)depth maxDepth:(int)maxDepth;
- (NSString *)role;
- (NSString *)description;
- (NSString *)identifier;
- (NSString *)value;
- (NSArray *)children;
@end

BOOL isChromiumBrowser(NSString *bundleId);
BOOL isSafari(NSString *bundleId);
BOOL isArc(NSString *bundleId);

void printAttributes(AXUIElementRef element, int indent, int maxDepth);
