#include <CoreFoundation/CoreFoundation.h>

#include "defaults.h"

CFStringRef to_cfstring(const char *c_string) {
  return CFStringCreateWithCString(/*alloc=*/kCFAllocatorDefault, /*cStr=*/c_string, /*encoding=*/kCFStringEncodingUTF8);
}

void defaults_set_bool(const char *app_id, const char *key, bool value) {
  // TODO Deallocate?
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFBooleanRef cf_value = value ? kCFBooleanTrue : kCFBooleanFalse;
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
}

bool defaults_sync(const char *app_id) {
  // TODO Deallocate?
  CFStringRef cf_app_id = to_cfstring(app_id);
  return CFPreferencesAppSynchronize(cf_app_id);
}
