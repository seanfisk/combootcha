#pragma once

#include <stdbool.h>

void defaults_set_bool(const char *app_id, const char *key, bool value);
bool defaults_sync(const char *app_id);
