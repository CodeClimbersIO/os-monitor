#import <Cocoa/Cocoa.h>
#import <Foundation/Foundation.h>
#import <QuartzCore/QuartzCore.h>

// Function to create a red border around the screen
// Parameters:
//   red - red color value (0.0 to 1.0)
//   green - green color value (0.0 to 1.0)
//   blue - blue color value (0.0 to 1.0)
//   width - width of the border in pixels
//   opacity - transparency of the border (0.0 to 1.0)
void create_border(double red, double green, double blue, double width,
                   double opacity);

// Function to remove the screen border
// Parameters:
//   borderWindow - pointer to the window to be removed
void remove_border(NSWindow *border_window);