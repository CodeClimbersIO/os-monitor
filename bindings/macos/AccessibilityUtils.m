#import "AccessibilityUtils.h"
#import <ApplicationServices/ApplicationServices.h>

NSString *getAXErrorDescription(AXError error) {
  switch (error) {
  case kAXErrorAttributeUnsupported:
    return @"The specified UI element does not support the specified attribute";
  case kAXErrorNoValue:
    return @"The specified attribute does not have a value";
  case kAXErrorIllegalArgument:
    return @"One or more of the arguments is an illegal value";
  case kAXErrorInvalidUIElement:
    return @"The UI element is invalid";
  case kAXErrorCannotComplete:
    return @"Cannot complete the operation (messaging failed or window might "
           @"be transitioning)";
  case kAXErrorNotImplemented:
    return @"The process does not fully support the accessibility API";
  case kAXErrorAPIDisabled:
    return @"Accessibility API is disabled";
  case kAXErrorFailure:
    return @"Operation failed";
  case kAXErrorNotificationUnsupported:
    return @"Notification not supported";
  default:
    return [NSString stringWithFormat:@"Unknown error code: %d", (int)error];
  }
}

BOOL has_accessibility_permissions(void) {
  NSDictionary *options = @{(__bridge id)kAXTrustedCheckOptionPrompt : @NO};
  return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

BOOL request_accessibility_permissions(void) {
  NSDictionary *options = @{(__bridge id)kAXTrustedCheckOptionPrompt : @YES};
  return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}