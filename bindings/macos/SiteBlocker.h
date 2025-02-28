#import <Cocoa/Cocoa.h>

BOOL start_site_blocking(const char **blocked_urls, int url_count,
                         const char *redirect_url);

void stop_site_blocking(void);

BOOL handle_enter_key_for_site_blocking(void);

BOOL redirect_to_vibes_page(void);

BOOL is_url_blocked(const char *url);

BOOL request_automation_permission(const char *bundle_id);