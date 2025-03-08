#import <Cocoa/Cocoa.h>
#import <CoreImage/CoreImage.h>
#import <Foundation/Foundation.h>

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

// Function to create a grayscale effect over the entire screen
// Parameters:
//   opacity - transparency of the effect (0.0 to 1.0)
void create_grayscale_effect(double opacity);

// Function to remove the grayscale effect
// Parameters:
//   grayscale_window - pointer to the window to be removed
void remove_grayscale_effect(NSWindow *grayscale_window);

// Function to create a false color effect over the entire screen
// Parameters:
//   opacity - transparency of the effect (0.0 to 1.0)
//   color0_r, color0_g, color0_b - RGB values for the first color (0.0 to 1.0)
//   color1_r, color1_g, color1_b - RGB values for the second color (0.0 to 1.0)
void create_false_color_effect(double opacity, double color0_r, double color0_g,
                               double color0_b, double color1_r,
                               double color1_g, double color1_b);

// Function to remove the false color effect
// Parameters:
//   false_color_window - pointer to the window to be removed
void remove_false_color_effect(NSWindow *false_color_window);