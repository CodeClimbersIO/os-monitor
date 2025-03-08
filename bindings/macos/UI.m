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

@interface GrayscaleWindow : NSWindow
@property(nonatomic, assign) CGFloat opacity;
@end

@implementation GrayscaleWindow

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

    // Allow events to pass through to windows below
    [self setIgnoresMouseEvents:YES];

    // Default opacity
    _opacity = 1.0;
  }
  return self;
}

- (void)applyGrayscaleEffect {
  // Create a visual effect view with grayscale filter
  NSVisualEffectView *visualEffectView =
      [[NSVisualEffectView alloc] initWithFrame:self.contentView.bounds];
  [visualEffectView
      setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];

  // Create a Core Image filter for grayscale
  CIFilter *grayscaleFilter = [CIFilter filterWithName:@"CIColorMonochrome"];
  [grayscaleFilter setDefaults];
  [grayscaleFilter setValue:[CIColor colorWithRed:0.7 green:0.7 blue:0.7]
                     forKey:@"inputColor"];
  [grayscaleFilter setValue:@1.0 forKey:@"inputIntensity"];

  // Apply the filter to the view
  visualEffectView.contentFilters = @[ grayscaleFilter ];

  // Set the alpha value
  [self setAlphaValue:self.opacity];

  // Replace the window's content view with our filtered view
  [self setContentView:visualEffectView];
}

@end

@interface FalseColorWindow : NSWindow
@property(nonatomic, assign) CGFloat opacity;
@property(nonatomic, strong) CIColor *color0;
@property(nonatomic, strong) CIColor *color1;
@end

@implementation FalseColorWindow

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

    // Allow events to pass through to windows below
    [self setIgnoresMouseEvents:YES];

    // Default values
    _opacity = 1.0;
    _color0 = [CIColor colorWithRed:1.0 green:1.0 blue:0.0]; // Yellow
    _color1 = [CIColor colorWithRed:0.0 green:0.0 blue:1.0]; // Blue
  }
  return self;
}

- (void)applyFalseColorEffect {
  // Create a visual effect view
  NSVisualEffectView *visualEffectView =
      [[NSVisualEffectView alloc] initWithFrame:self.contentView.bounds];
  [visualEffectView
      setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];

  // Create a Core Image filter for false color
  CIFilter *falseColorFilter = [CIFilter filterWithName:@"CIFalseColor"];
  [falseColorFilter setDefaults];
  [falseColorFilter setValue:self.color0 forKey:@"inputColor0"];
  [falseColorFilter setValue:self.color1 forKey:@"inputColor1"];

  // Apply the filter to the view
  visualEffectView.contentFilters = @[ falseColorFilter ];

  // Set the alpha value
  [self setAlphaValue:self.opacity];

  // Replace the window's content view with our filtered view
  [self setContentView:visualEffectView];
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

// Function to create a grayscale effect over the screen
void create_grayscale_effect(double opacity) {
  @autoreleasepool {
    @try {
      NSLog(@"Creating grayscale effect with opacity: %.2f", opacity);

      [NSApplication sharedApplication];
      [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];

      NSScreen *mainScreen = [NSScreen mainScreen];
      NSRect screenFrame = [mainScreen frame];

      GrayscaleWindow *grayscaleWindow = [[GrayscaleWindow alloc]
          initWithContentRect:screenFrame
                    styleMask:NSWindowStyleMaskBorderless
                      backing:NSBackingStoreBuffered
                        defer:NO];

      grayscaleWindow.opacity = opacity;
      [grayscaleWindow applyGrayscaleEffect];
      [grayscaleWindow makeKeyAndOrderFront:nil];

    } @catch (NSException *exception) {
      NSLog(@"Exception caught: %@", exception);
      NSLog(@"Reason: %@", [exception reason]);
      NSLog(@"Stack trace: %@", [exception callStackSymbols]);
    } @finally {
      NSLog(@"Grayscale effect creation attempt completed");
    }
  }
}

// Function to remove the grayscale effect
void remove_grayscale_effect(NSWindow *grayscale_window) {
  if (grayscale_window != nil) {
    [grayscale_window close];
  }
}

// Function to create a false color effect over the screen
void create_false_color_effect(double opacity, double color0_r, double color0_g,
                               double color0_b, double color1_r,
                               double color1_g, double color1_b) {
  @autoreleasepool {
    @try {
      NSLog(@"Creating false color effect with opacity: %.2f", opacity);
      NSLog(@"Color0: RGB(%.2f, %.2f, %.2f), Color1: RGB(%.2f, %.2f, %.2f)",
            color0_r, color0_g, color0_b, color1_r, color1_g, color1_b);

      [NSApplication sharedApplication];
      [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];

      NSScreen *mainScreen = [NSScreen mainScreen];
      NSRect screenFrame = [mainScreen frame];

      FalseColorWindow *falseColorWindow = [[FalseColorWindow alloc]
          initWithContentRect:screenFrame
                    styleMask:NSWindowStyleMaskBorderless
                      backing:NSBackingStoreBuffered
                        defer:NO];

      falseColorWindow.opacity = opacity;
      falseColorWindow.color0 = [CIColor colorWithRed:color0_r
                                                green:color0_g
                                                 blue:color0_b];
      falseColorWindow.color1 = [CIColor colorWithRed:color1_r
                                                green:color1_g
                                                 blue:color1_b];

      [falseColorWindow applyFalseColorEffect];
      [falseColorWindow makeKeyAndOrderFront:nil];

    } @catch (NSException *exception) {
      NSLog(@"Exception caught: %@", exception);
      NSLog(@"Reason: %@", [exception reason]);
      NSLog(@"Stack trace: %@", [exception callStackSymbols]);
    } @finally {
      NSLog(@"False color effect creation attempt completed");
    }
  }
}

// Function to remove the false color effect
void remove_false_color_effect(NSWindow *false_color_window) {
  if (false_color_window != nil) {
    [false_color_window close];
  }
}
