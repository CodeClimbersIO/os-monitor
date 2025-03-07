#import "UI.h"

@interface BorderWindow : NSWindow
@property(nonatomic, strong) NSColor *borderColor;
@property(nonatomic, assign) CGFloat borderWidth;
@property(nonatomic, assign) CGFloat borderOpacity;
@end

@implementation BorderWindow

- (instancetype)initWithContentRect:(NSRect)contentRect
                          styleMask:(NSWindowStyleMask)style
                            backing:(NSBackingStoreType)backingStoreType
                              defer:(BOOL)flag {
  self = [super initWithContentRect:contentRect
                          styleMask:NSWindowStyleMaskBorderless
                            backing:backingStoreType
                              defer:flag];
  if (self) {
    [self setBackgroundColor:[NSColor clearColor]];
    [self setOpaque:NO];
    [self setLevel:NSStatusWindowLevel];
    [self setIgnoresMouseEvents:YES];

    // Default values
    _borderColor = [NSColor redColor];
    _borderWidth = 5.0;
    _borderOpacity = 1.0;
  }
  return self;
}

- (void)drawBorder {
  [self setAlphaValue:self.borderOpacity];
  [self.contentView setWantsLayer:YES];
  self.contentView.layer.borderWidth = self.borderWidth;
  self.contentView.layer.borderColor = self.borderColor.CGColor;
}

@end

// Function to create a red border around the screen
void create_border(double red, double green, double blue, double width,
                   double opacity) {
  @autoreleasepool {
    @try {
      // print out thread name
      NSLog(@"Thread name: %@", [NSThread currentThread]);
      NSLog(@"Creating border with color RGB(%.2f, %.2f, %.2f), width: %.2f, "
            @"opacity: %.2f",
            red, green, blue, width, opacity);

      [NSApplication sharedApplication];
      [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];

      NSScreen *mainScreen = [NSScreen mainScreen];
      NSRect screenFrame = [mainScreen frame];

      BorderWindow *borderWindow =
          [[BorderWindow alloc] initWithContentRect:screenFrame
                                          styleMask:NSWindowStyleMaskBorderless
                                            backing:NSBackingStoreBuffered
                                              defer:NO];

      borderWindow.borderColor = [NSColor colorWithRed:red
                                                 green:green
                                                  blue:blue
                                                 alpha:1.0];
      borderWindow.borderWidth = width;
      borderWindow.borderOpacity = opacity;

      [borderWindow drawBorder];
      [borderWindow makeKeyAndOrderFront:nil];

    } @catch (NSException *exception) {
      NSLog(@"Exception caught: %@", exception);
      NSLog(@"Reason: %@", [exception reason]);
      NSLog(@"Stack trace: %@", [exception callStackSymbols]);
    } @finally {
      NSLog(@"Border creation attempt completed");
    }
  }
}

// Function to remove the screen border
void remove_border(NSWindow *border_window) {
  if (border_window != nil) {
    [border_window close];
  }
}
