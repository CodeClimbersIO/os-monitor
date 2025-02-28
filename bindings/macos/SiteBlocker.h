#import <Cocoa/Cocoa.h>

// Function to start site blocking with a list of URLs to block and a redirect URL
BOOL start_site_blocking(const char** blocked_urls, int url_count, const char* redirect_url);

// Function to stop site blocking
void stop_site_blocking(void);

// Function to handle an Enter key press and check for blocked sites
BOOL handle_enter_key_for_site_blocking(void);

// Function to redirect a browser to our "vibes" page
BOOL redirect_to_vibes_page(void);

// Check if a URL is in the blocked list
BOOL is_url_blocked(const char* url);

// Add this function declaration to SiteBlocker.h
BOOL request_automation_permission(const char* bundle_id); 