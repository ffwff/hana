#pragma once
#include "array.h"
#include <stdint.h>
#include <stdbool.h>

struct string;
struct string *string_malloc(const char *str);
struct string *string_append(const struct string *left, const struct string *right);
struct string *string_repeat(const struct string *left, int64_t n);
bool string_is_empty(const struct string *str);
int string_cmp(const struct string *left, const struct string *right);
struct string *string_at(const struct string *str, int64_t n);