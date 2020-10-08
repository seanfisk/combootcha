#pragma once

#include <MacTypes.h>

void user_defaults_set_bool(const char *app_id, const char *key, bool value);
void user_defaults_set_i64(const char *app_id, const char *key, SInt64 value);
void user_defaults_set_f64(const char *app_id, const char *key, Float64 value);
void user_defaults_set_string(const char *app_id, const char *key, const char *value);
bool user_defaults_sync(const char *app_id);
