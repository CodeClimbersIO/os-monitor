#import "SiteBlocker.h"
#import "WindowUtils.h"
#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>

// Static variables to store state
static BOOL siteBlockingEnabled = NO;
static NSMutableArray<NSString*> *blockedUrls = nil;
static NSString *vibesUrl = @"https://ebb.cool/vibes";

BOOL start_site_blocking(const char** blocked_urls, int url_count) {
    NSLog(@"start_site_blocking");
    @autoreleasepool {
        if (blockedUrls == nil) {
            blockedUrls = [NSMutableArray array];
        } else {
            [blockedUrls removeAllObjects];
        }
        
        // Copy the URLs from C strings to NSString objects
        for (int i = 0; i < url_count; i++) {
            if (blocked_urls[i]) {
                NSString *url = [NSString stringWithUTF8String:blocked_urls[i]];
                [blockedUrls addObject:url];
                NSLog(@"Blocking URL: %@", url);
            }
        }
        
        siteBlockingEnabled = YES;
        NSLog(@"Site blocking enabled");
        return YES;
    }
}

void stop_site_blocking(void) {
    siteBlockingEnabled = NO;
    NSLog(@"Site blocking disabled");
}

BOOL is_url_blocked(const char* url) {
    NSLog(@"is_url_blocked");
    NSLog(@"url: %s", url);
    if (!url || !siteBlockingEnabled || blockedUrls.count == 0) {
        return NO;
    }
    
    @autoreleasepool {
        NSString *currentUrl = [NSString stringWithUTF8String:url];
        
        // Check if the current URL contains any of the blocked URLs
        for (NSString *blockedUrl in blockedUrls) {
            NSLog(@"blockedUrl: %@", blockedUrl);
            NSLog(@"currentUrl: %@", currentUrl);
            if ([currentUrl rangeOfString:blockedUrl options:NSCaseInsensitiveSearch].location != NSNotFound) {
                NSLog(@"URL %@ is blocked (matched %@)", currentUrl, blockedUrl);
                return YES;
                
            }
        }
        
        return NO;
    }
}

BOOL redirect_to_vibes_page(void) {
    @autoreleasepool {
        NSRunningApplication *frontApp = get_frontmost_app();
        if (!frontApp) {
            NSLog(@"Failed to get frontmost application");
            return NO;
        }
        
        NSString *bundleId = frontApp.bundleIdentifier;
        
        // Create AppleScript to redirect based on browser
        NSString *appleScript = nil;
        
        if ([bundleId isEqualToString:@"com.apple.Safari"]) {
            appleScript = [NSString stringWithFormat:@"tell application \"Safari\" to set URL of current tab of front window to \"%@\"", vibesUrl];
        } 
        else if ([bundleId isEqualToString:@"com.google.Chrome"] || 
                 [bundleId isEqualToString:@"com.google.Chrome.canary"] || 
                 [bundleId isEqualToString:@"com.google.Chrome.beta"]) {
            appleScript = [NSString stringWithFormat:@"tell application \"Google Chrome\" to set URL of active tab of front window to \"%@\"", vibesUrl];
        }
        else if ([bundleId isEqualToString:@"com.microsoft.Edge"]) {
            appleScript = [NSString stringWithFormat:@"tell application \"Microsoft Edge\" to set URL of active tab of front window to \"%@\"", vibesUrl];
        }
        else if ([bundleId isEqualToString:@"company.thebrowser.Browser"]) {
            appleScript = [NSString stringWithFormat:@"tell application \"Arc\" to set URL of active tab of front window to \"%@\"", vibesUrl];
        }
        else {
            NSLog(@"Unsupported browser: %@", bundleId);
            return NO;
        }
        
        // Execute the AppleScript
        NSAppleScript *script = [[NSAppleScript alloc] initWithSource:appleScript];
        NSDictionary *error = nil;
        [script executeAndReturnError:&error];
        
        if (error) {
            NSLog(@"AppleScript error: %@", error);
            return NO;
        }
        
        NSLog(@"Redirected to vibes page");
        return YES;
    }
}

BOOL handle_enter_key_for_site_blocking(void) {
    NSLog(@"handle_enter_key_for_site_blocking");
    if (!siteBlockingEnabled) {
        return NO;
    }

    NSLog(@"handle_enter_key_for_site_blocking start");
    NSLog(@"try detect before");

    @autoreleasepool {
        // Use a global concurrent queue instead of the main queue
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.5 * NSEC_PER_SEC)), 
                       dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            NSLog(@"try detect");
            WindowTitle* windowInfo = detect_focused_window();
            if (!windowInfo) {
                NSLog(@"detect_focused_window no window info");
                return;
            }
            NSLog(@"detect_focused_window end");
            
            // Check if it's a browser and has a URL
            if (!windowInfo->url) {
                free((void*)windowInfo->app_name);
                free((void*)windowInfo->window_title);
                free((void*)windowInfo->bundle_id);
                NSLog(@"No URL found");
                return;
            }

            NSLog(@"URL: %s", windowInfo->url);
            // Check if the URL is blocked
            BOOL isBlocked = is_url_blocked(windowInfo->url);
            
            // Free the window info
            free((void*)windowInfo->app_name);
            free((void*)windowInfo->window_title);
            free((void*)windowInfo->bundle_id);
            free((void*)windowInfo->url);
            
            // If URL is blocked, redirect to vibes page
            if (isBlocked) {
                redirect_to_vibes_page();
            }
        });
        
        return YES; // Return YES to indicate we've scheduled a check
    }
} 