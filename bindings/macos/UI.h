#import <Cocoa/Cocoa.h>
#import <CoreImage/CoreImage.h>
#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>

// Function to create a grayscale effect over the entire screen
// Parameters:
//   opacity - transparency of the effect (0.0 to 1.0)
void create_grayscale_effect(double opacity);

// Function to remove the grayscale effect
// Parameters:
//   grayscale_window - pointer to the window to be removed
void remove_grayscale_effect(NSWindow *grayscale_window);