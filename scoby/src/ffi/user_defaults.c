#include <CoreFoundation/CFBase.h>
#include <CoreFoundation/CFData.h>
#include <CoreFoundation/CFNumber.h>
#include <CoreFoundation/CFPreferences.h>

#include "user_defaults.h"

CFStringRef to_cfstring(const char *c_string) {
  // TODO This function can return NULL "if there was a problem creating the
  // object"
  return CFStringCreateWithCStringNoCopy(
      /*alloc=*/kCFAllocatorDefault, /*cStr=*/c_string,
      /*encoding=*/kCFStringEncodingUTF8,
      /*contentsDeallocator=*/kCFAllocatorNull // We'll deallocate the C string
  );
}

void user_defaults_set_bool(const char *app_id, const char *key, bool value) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFBooleanRef cf_value = value ? kCFBooleanTrue : kCFBooleanFalse;
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
  CFRelease(cf_app_id);
  CFRelease(cf_key);
}

void user_defaults_set_i64(const char *app_id, const char *key, SInt64 value) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFNumberRef cf_value =
      CFNumberCreate(/*allocator=*/kCFAllocatorDefault,
                     /*theType=*/kCFNumberSInt64Type, /*valuePtr=*/&value);
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
  CFRelease(cf_app_id);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

void user_defaults_set_f64(const char *app_id, const char *key, Float64 value) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFNumberRef cf_value =
      CFNumberCreate(/*allocator=*/kCFAllocatorDefault,
                     /*theType=*/kCFNumberFloat64Type, /*valuePtr=*/&value);
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
  CFRelease(cf_app_id);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

void user_defaults_set_string(const char *app_id, const char *key,
                              const char *value) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFStringRef cf_value = to_cfstring(value);
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
  CFRelease(cf_app_id);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

void user_defaults_set_dict(const char *app_id, const char *key,
                            CFDictionaryRef value) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFPreferencesSetAppValue(cf_key, value, cf_app_id);
  CFRelease(cf_app_id);
  CFRelease(cf_key);
}

void user_defaults_set_data(const char *app_id, const char *key,
                            const UInt8 *bytes, const long length) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  CFStringRef cf_key = to_cfstring(key);
  CFDataRef cf_value = CFDataCreate(/*allocator=*/kCFAllocatorDefault,
                                    /*bytes=*/bytes, /*length=*/length);
  CFPreferencesSetAppValue(cf_key, cf_value, cf_app_id);
  CFRelease(cf_app_id);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

bool user_defaults_sync(const char *app_id) {
  CFStringRef cf_app_id = to_cfstring(app_id);
  bool success = CFPreferencesAppSynchronize(cf_app_id);
  CFRelease(cf_app_id);
  return success;
}

CFMutableDictionaryRef user_defaults_dict_create() {
  return CFDictionaryCreateMutable(
      /*allocator=*/kCFAllocatorDefault,
      /*capacity=*/0, // unlimited capacity,
      // Since we are using only CFTypes, the default callback functions should
      // suffice. See
      // https://developer.apple.com/documentation/corefoundation/1516791-cfdictionarycreatemutable?language=objc
      /*keyCallbacks=*/&kCFTypeDictionaryKeyCallBacks,
      /*valueCallbacks=*/&kCFTypeDictionaryValueCallBacks);
}

void user_defaults_dict_set_bool_value(CFMutableDictionaryRef dict,
                                       const char *key, bool value) {
  CFStringRef cf_key = to_cfstring(key);
  CFBooleanRef cf_value = value ? kCFBooleanTrue : kCFBooleanFalse;
  CFDictionarySetValue(dict, cf_key, cf_value);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

void user_defaults_dict_set_i64_value(CFMutableDictionaryRef dict,
                                      const char *key, SInt64 value) {
  CFStringRef cf_key = to_cfstring(key);
  CFNumberRef cf_value =
      CFNumberCreate(/*allocator=*/kCFAllocatorDefault,
                     /*theType=*/kCFNumberSInt64Type, /*valuePtr=*/&value);
  CFDictionarySetValue(dict, cf_key, cf_value);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

void user_defaults_dict_set_f64_value(CFMutableDictionaryRef dict,
                                      const char *key, Float64 value) {
  CFStringRef cf_key = to_cfstring(key);
  CFNumberRef cf_value =
      CFNumberCreate(/*allocator=*/kCFAllocatorDefault,
                     /*theType=*/kCFNumberFloat64Type, /*valuePtr=*/&value);
  CFDictionarySetValue(dict, cf_key, cf_value);
  CFRelease(cf_key);
  CFRelease(cf_value);
}

void user_defaults_dict_release(CFMutableDictionaryRef dict) {
  CFRelease(dict);
}
