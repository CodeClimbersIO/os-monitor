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

/**
 * Find the URL element in the given accessibility element. Recursively searches
 * through children. Assumes that the URL element is a static text or a text
 * field.
 * @param element The accessibility element to search
 * @return The URL element if found, otherwise nil
 */
- (AccessibilityElement *)findUrlElement {
  if (!self)
    return nil;

  NSString *role = [self role];

  if ([role isEqualToString:NSAccessibilityStaticTextRole] ||
      [role isEqualToString:NSAccessibilityTextFieldRole]) {
    NSString *value = [self value];
    if (value && isDomain(value)) {
      return self;
    }
  }

  NSArray *children = [self children];
  for (id child in children) {
    AccessibilityElement *childElement = [[AccessibilityElement alloc]
        initWithAXUIElement:(__bridge AXUIElementRef)child];
    AccessibilityElement *urlElement = [childElement findUrlElement];
    if (urlElement != nil) {
      return urlElement;
    }
  }

  return nil;
}

/**
 * Find the URL field in the given accessibility element. Recursively searches
 * through children. Meant to be used to find the address bar in a browser.
 * @param bundleId The bundle ID of the browser
 * @return The URL field if found, otherwise nil
 */
- (AccessibilityElement *)findAddressBarForBrowser:(NSString *)bundleId {
  if (!_axUIElement)
    return nil;

  // Get the role
  NSString *role = [self role];

  // Print attributes for debugging
  [self printAttributes];

  // Use description approach for Chromium-based browsers
  if (isChromiumBrowser(bundleId)) {
    if ([role isEqualToString:NSAccessibilityTextFieldRole]) {
      // Get the description
      NSString *description = [self description];

      if (description) {
        // Check if this is a URL field based on its description
        NSArray *urlIdentifiers = @[
          @"Address", @"URL", @"Location", @"Address and search bar",
          @"address field"
        ];
        for (NSString *identifier in urlIdentifiers) {
          if ([description rangeOfString:identifier
                                 options:NSCaseInsensitiveSearch]
                  .location != NSNotFound) {
            return
                [[AccessibilityElement alloc] initWithAXUIElement:_axUIElement];
          }
        }
      }
    }
  } else if (isSafari(bundleId)) {
    // Safari-specific approach
    NSString *value = [self value];

    // Check if the value looks like a URL
    if (value &&
        ([value hasPrefix:@"http://"] || [value hasPrefix:@"https://"] ||
         [value hasPrefix:@"www."] || [value containsString:@"."])) {
      return [[AccessibilityElement alloc] initWithAXUIElement:_axUIElement];
    }
  } else if (isArc(bundleId)) {
    NSLog(@"Arc browser detected");

    // Check for the identifier attribute
    NSString *identifier = [self identifier];

    // Check if this is the URL field based on the identifier
    if ([identifier isEqualToString:@"commandBarPlaceholderTextField"]) {
      return [[AccessibilityElement alloc] initWithAXUIElement:_axUIElement];
    }
  }

  // Recursively search children
  NSArray *children = [self children];
  for (id child in children) {
    AccessibilityElement *childElement = [[AccessibilityElement alloc]
        initWithAXUIElement:(__bridge AXUIElementRef)child];
    AccessibilityElement *urlField =
        [childElement findAddressBarForBrowser:bundleId];
    if (urlField) {
      return urlField;
    }
  }

  return nil;
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

@end
