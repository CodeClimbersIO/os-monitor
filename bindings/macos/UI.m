#import "UI.h"
#import "AccessibilityElement.h"
#import "Application.h"

static AppWindow *currentTypewriterWindow = nil;

@interface AppWindow : NSWindow
@property(nonatomic, assign) CGFloat opacity;
@end

@implementation AppWindow

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

- (void)syncOrderWithFocusedWindow {
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
  [self orderWindow:NSWindowBelow relativeTo:windowId];
}
@end

void create_typewriter_window(double opacity) {
  @try {

    // Create grayscale window covering the entire screen
    NSScreen *mainScreen = [NSScreen mainScreen];
    NSRect screenFrame = [mainScreen frame];

    AppWindow *grayscaleWindow =
        [[AppWindow alloc] initWithContentRect:screenFrame
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
    [grayscaleWindow syncOrderWithFocusedWindow];

    currentTypewriterWindow = grayscaleWindow;

  } @catch (NSException *exception) {
    NSLog(
        @"Exception in create_typewriter_window: %@\nReason: %@\nCallStack: %@",
        exception.name, exception.reason, [exception callStackSymbols]);
  }
}

void remove_typewriter_window() {
  if (currentTypewriterWindow) {
    [currentTypewriterWindow close];
    currentTypewriterWindow = nil;
  }
}

void sync_typewriter_window_order() {
  if (currentTypewriterWindow) {
    [currentTypewriterWindow syncOrderWithFocusedWindow];
  }
}

// void sync_typewriter_window_order(AppWindow *grayscale_window) {
//       // Get the frontmost app using our existing FocusedApp class
//     FocusedApp *frontmostApp = [FocusedApp frontmostApp];
//     if (!frontmostApp) {
//       NSLog(@"Failed to get frontmost application");
//       return;
//     }

//     // Get the window ID of the focused window
//     CGWindowID windowId = [frontmostApp getFocusedWindowId];
//     if (windowId == kCGNullWindowID) {
//       NSLog(@"Failed to get window ID of focused window");
//       return;
//     }

//     NSLog(@"Creating grayscale effect for window ID: %u", windowId);
//   if (grayscale_window != nil) {
//     [grayscale_window orderWindow:NSWindowBelow relativeTo:windowId];
//   }
// }
