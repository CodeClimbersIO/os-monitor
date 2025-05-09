#import "AccessibilityUtils.h"
#import "Application.h"
#import "Blocker.h"
#import <Cocoa/Cocoa.h>

typedef NS_ENUM(int32_t, MouseEventType) {
  MouseEventTypeMove = 0,
  MouseEventTypeLeftDown,
  MouseEventTypeLeftUp,
  MouseEventTypeRightDown,
  MouseEventTypeRightUp,
  MouseEventTypeMiddleDown,
  MouseEventTypeMiddleUp,
  MouseEventTypeScroll
};

typedef NS_ENUM(int32_t, WindowEventType) {
  WindowEventTypeFocused,
  WindowEventTypeTitleChanged
};
typedef void (*MouseEventCallback)(double x, double y, int32_t eventType,
                                   int32_t scrollDelta);
typedef void (*KeyboardEventCallback)(int32_t keyCode);
typedef void (*WindowEventCallback)(int32_t windowNumber, int32_t x, int32_t y,
                                    int32_t width, int32_t height,
                                    const char *title, const char *url,
                                    const char *bundle_id, int32_t eventType);

const char *get_app_icon_data(const char *bundle_id);
void free_icon_data(const char *data);
void start_monitoring(MouseEventCallback mouseCallback,
                      KeyboardEventCallback keyboardCallback);
void create_screen_border(double red, double green, double blue, double width,
                          double opacity);
void remove_screen_border(NSWindow *border_window);
void create_screen_false_color(double opacity, double color0_r, double color0_g,
                               double color0_b, double color1_r,
                               double color1_g, double color1_b);
void remove_screen_false_color(NSWindow *false_color_window);
