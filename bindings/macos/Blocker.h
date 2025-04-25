#import "AccessibilityElement.h"
#import "Application.h"
#import "UI.h"
#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>

typedef void (*AppBlockedCallback)(const char **app_names,
                                   const char **bundle_ids, int count);

BOOL start_blocking(const char **blocked_urls, int url_count,
                    const char *redirect_url, BOOL blocklist_mode);

void stop_blocking(void);

BOOL handle_enter_key_for_site_blocking(void);

BOOL redirect_to_vibes_page(void);

BOOL is_blocked(const char *external_app_id);

BOOL close_app(const char *bundle_id, const bool send_callback);

void register_app_blocked_callback(AppBlockedCallback callback);