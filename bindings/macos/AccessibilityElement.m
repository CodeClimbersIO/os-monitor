#import "AccessibilityElement.h"

@implementation AccessibilityElement

- (instancetype)initWithAXUIElement:(AXUIElementRef)element {
  self = [super init];
  if (self) {
    _axUIElement = element;
    CFRetain(_axUIElement);
  }
  return self;
}

- (void)dealloc {
  if (_axUIElement) {
    CFRelease(_axUIElement);
  }
}

- (void)focus {
  AXUIElementSetAttributeValue(_axUIElement, kAXFocusedAttribute,
                               kCFBooleanTrue);
}

- (void)printAttributesWithDepth:(int)depth maxDepth:(int)maxDepth {
  if (!_axUIElement)
    return;
  if (depth > maxDepth)
    return;

  CFArrayRef attributeNames;
  AXUIElementCopyAttributeNames(_axUIElement, &attributeNames);
  NSArray *attributes = (__bridge_transfer NSArray *)attributeNames;

  CFStringRef titleRef;
  AXUIElementCopyAttributeValue(_axUIElement, kAXTitleAttribute,
                                (CFTypeRef *)&titleRef);
  NSString *title = (__bridge_transfer NSString *)titleRef;

  CFStringRef roleRef;
  AXUIElementCopyAttributeValue(_axUIElement, kAXRoleAttribute,
                                (CFTypeRef *)&roleRef);
  NSString *role = (__bridge_transfer NSString *)roleRef;

  // Create indent based on depth with colors
  char indent[100] = "";
  // ANSI foreground color codes from 31-36 (red, green, cyan, blue, magenta,
  // yellow)
  int colorCode = 31 + (depth % 6);
  for (int i = 0; i < depth; i++) {
    strcat(indent, "  "); // Just add spaces without color
  }

  // Add color code at the start of the line, but after the indent
  char colorStart[20];
  sprintf(colorStart, "\033[%dm", colorCode);

  // Reset color code at the end of indent
  char resetColor[] = "\033[0m";

  printf("\n%s%s=== Element at depth %d ===%s\n", indent, colorStart, depth,
         resetColor);
  printf("%s%sRole: %s%s\n", indent, colorStart, [role UTF8String], resetColor);
  if (title) {
    printf("%s%sTitle: %s%s\n", indent, colorStart, [title UTF8String],
           resetColor);
  }

  for (NSString *attribute in attributes) {
    CFTypeRef valueRef;
    AXError error = AXUIElementCopyAttributeValue(
        _axUIElement, (__bridge CFStringRef)attribute, &valueRef);

    if (error == kAXErrorSuccess) {
      id value = (__bridge_transfer id)valueRef;
      printf("%s%sAttribute: %s = %s%s\n", indent, colorStart,
             [attribute UTF8String], [[value description] UTF8String],
             resetColor);

      // Recursively explore children
      if ([attribute isEqualToString:NSAccessibilityChildrenAttribute]) {
        NSArray *children = (NSArray *)value;
        for (id child in children) {
          AccessibilityElement *childElement = [[AccessibilityElement alloc]
              initWithAXUIElement:(__bridge AXUIElementRef)child];
          [childElement printAttributesWithDepth:depth + 1 maxDepth:maxDepth];
        }
      }
    } else {
      printf("%s%sAttribute: %s (Error getting value: %d)%s\n", indent,
             colorStart, [attribute UTF8String], error, resetColor);
    }
  }
  printf("%s%s===========================%s\n\n", indent, colorStart,
         resetColor);
}

- (void)printAttributes {
  [self printAttributesWithDepth:0 maxDepth:3];
}

- (NSString *)role {
  CFStringRef roleRef;
  if (AXUIElementCopyAttributeValue(_axUIElement, kAXRoleAttribute,
                                    (CFTypeRef *)&roleRef) == kAXErrorSuccess) {
    return (__bridge_transfer NSString *)roleRef;
  }
  return nil;
}

- (NSString *)description {
  CFStringRef descriptionRef;
  if (AXUIElementCopyAttributeValue(_axUIElement, kAXDescriptionAttribute,
                                    (CFTypeRef *)&descriptionRef) ==
      kAXErrorSuccess) {
    return (__bridge_transfer NSString *)descriptionRef;
  }
  return nil;
}

- (NSString *)identifier {
  CFStringRef identifierRef;
  if (AXUIElementCopyAttributeValue(_axUIElement, kAXIdentifierAttribute,
                                    (CFTypeRef *)&identifierRef) ==
      kAXErrorSuccess) {
    return (__bridge_transfer NSString *)identifierRef;
  }
  return nil;
}

- (NSString *)value {
  CFStringRef valueRef;
  if (AXUIElementCopyAttributeValue(_axUIElement, kAXValueAttribute,
                                    (CFTypeRef *)&valueRef) ==
      kAXErrorSuccess) {
    return (__bridge_transfer NSString *)valueRef;
  }
  return nil;
}

- (NSArray *)children {
  CFArrayRef childrenRef;
  if (AXUIElementCopyAttributeValue(_axUIElement, kAXChildrenAttribute,
                                    (CFTypeRef *)&childrenRef) ==
      kAXErrorSuccess) {
    return (__bridge_transfer NSArray *)childrenRef;
  }
  return @[];
}

- (NSString *)title {
  CFStringRef titleRef;
  if (AXUIElementCopyAttributeValue(_axUIElement, kAXTitleAttribute,
                                    (CFTypeRef *)&titleRef) ==
      kAXErrorSuccess) {
    return (__bridge_transfer NSString *)titleRef;
  }
  return nil;
}

- (AXError)setValue:(NSString *)value {
  return AXUIElementSetAttributeValue(_axUIElement, kAXValueAttribute,
                                      (__bridge CFTypeRef)value);
}

- (AccessibilityElement *)valueForAttribute:(CFStringRef)attribute {
  CFTypeRef valueRef;
  AXError error =
      AXUIElementCopyAttributeValue(_axUIElement, attribute, &valueRef);

  if (error == kAXErrorSuccess) {
    if (CFGetTypeID(valueRef) == AXUIElementGetTypeID()) {
      AccessibilityElement *element = [[AccessibilityElement alloc]
          initWithAXUIElement:(AXUIElementRef)valueRef];
      CFRelease(valueRef);
      return element;
    } else {
      CFRelease(valueRef);
    }
  }

  return nil;
}

- (BOOL)addObserver:(AXObserverRef)observer
       notification:(CFStringRef)notification
           callback:(AXObserverCallback)callback
           userData:(void *)userData {
  if (!_axUIElement || !observer) {
    return NO;
  }

  AXError error =
      AXObserverAddNotification(observer, _axUIElement, notification, userData);
  return (error == kAXErrorSuccess);
}

- (void)removeObserver:(AXObserverRef)observer
          notification:(CFStringRef)notification {
  if (_axUIElement && observer) {
    AXObserverRemoveNotification(observer, _axUIElement, notification);
  }
}

@end
