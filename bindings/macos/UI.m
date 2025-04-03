#import "UI.h"
#import "AccessibilityElement.h"
#import "Application.h"

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
    [self setLevel:NSNormalWindowLevel];

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

// Function to create a grayscale effect over the screen
void create_grayscale_effect(double opacity) {
  @try {
    // Get the frontmost app using our existing FocusedApp class
    FocusedApp *frontmostApp = [FocusedApp frontmostApp];
    if (!frontmostApp) {
      NSLog(@"Failed to get frontmost application");
      return;
    }

    // Get the window ID of the focused window
    CGWindowID windowId = [frontmostApp getFocusedWindowId];
    if (windowId == kCGNullWindowID) {
      NSLog(@"Failed to get window ID of focused window");
      return;
    }

    NSLog(@"Creating grayscale effect for window ID: %u", windowId);

    // Create grayscale window covering the entire screen
    NSScreen *mainScreen = [NSScreen mainScreen];
    NSRect screenFrame = [mainScreen frame];

    GrayscaleWindow *grayscaleWindow =
        [[GrayscaleWindow alloc] initWithContentRect:screenFrame
                                           styleMask:NSWindowStyleMaskBorderless
                                             backing:NSBackingStoreBuffered
                                               defer:NO];

    // Configure grayscale effect
    grayscaleWindow.opacity = opacity;
    [grayscaleWindow applyGrayscaleEffect];

    // Set window level and show it
    [grayscaleWindow setLevel:NSNormalWindowLevel];
    [grayscaleWindow makeKeyAndOrderFront:nil];

    // Order it below the focused window
    [grayscaleWindow orderWindow:NSWindowBelow relativeTo:windowId];

  } @catch (NSException *exception) {
    NSLog(
        @"Exception in create_grayscale_effect: %@\nReason: %@\nCallStack: %@",
        exception.name, exception.reason, [exception callStackSymbols]);
  }
}

// Function to remove the grayscale effect
void remove_grayscale_effect(NSWindow *grayscale_window) {
  if (grayscale_window != nil) {
    [grayscale_window close];
  }
}