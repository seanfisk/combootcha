#pragma once

#include <CoreFoundation/CFDictionary.h>
#include <MacTypes.h>

void user_defaults_set_bool(const char *app_id, const char *key, bool value);
void user_defaults_set_i64(const char *app_id, const char *key, SInt64 value);
void user_defaults_set_f64(const char *app_id, const char *key, Float64 value);
void user_defaults_set_string(const char *app_id, const char *key,
                              const char *value);
void user_defaults_set_dict(const char *app_id, const char *key,
                            CFDictionaryRef value);
void user_defaults_set_data(
    const char *app_id, const char *key, const UInt8 *bytes,
    const long length // CFIndex is typedef'd to long but don't want to import
                      // the whole header
);
bool user_defaults_sync(const char *app_id);

CFMutableDictionaryRef user_defaults_dict_create();
void user_defaults_dict_set_bool_value(CFMutableDictionaryRef dict,
                                       const char *key, bool value);
void user_defaults_dict_set_i64_value(CFMutableDictionaryRef dict,
                                      const char *key, SInt64 value);
void user_defaults_dict_set_f64_value(CFMutableDictionaryRef dict,
                                      const char *key, Float64 value);
void user_defaults_dict_set_string_value(CFMutableDictionaryRef dict,
                                         const char *key, const char *value);
void user_defaults_dict_release(CFMutableDictionaryRef dict);
