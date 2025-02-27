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
        
        // Execute the AppleScript asynchronously
        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            NSLog(@"Executing AppleScript on background thread");
            NSAppleScript *script = [[NSAppleScript alloc] initWithSource:appleScript];
            NSDictionary *error = nil;
            [script executeAndReturnError:&error];
            
            if (error) {
                NSLog(@"AppleScript error: %@", error);
            } else {
                NSLog(@"Redirected to vibes page");
            }
        });
        
        // Return immediately while the script executes in the background
        return YES;
    }
} 