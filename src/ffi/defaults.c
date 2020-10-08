#include <CoreFoundation/CoreFoundation.h>

#include "defaults.h"

void defaults_set_bool(const char *app_id, const char *key, bool value) {
  CFStringRef cf_app_id = CFStringCreateWithCString(/*alloc=*/kCFAllocatorDefault, /*cStr=*/app_id, /*encoding=*/kCFStringEncodingUTF8);
  CFStringRef cf_key = CFStringCreateWithCString(/*alloc=*/kCFAllocatorDefault, /*cStr=*/key, /*encoding=*/kCFStringEncodingUTF8);
  CFBooleanRef cf_value = value ? kCFBooleanTrue : kCFBooleanFalse;
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
}
