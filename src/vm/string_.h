#pragma once
#include "array_obj.h"
#include <stdint.h>
#include <stdbool.h>

struct string;
struct vm;
struct string *string_malloc(const char *str, const struct vm *);
struct string *string_append(const struct string *left, const struct string *right, const struct vm *);
void string_append_in_place(struct string *left, const struct string *right);
struct string *string_repeat(const struct string *left, int64_t n, const struct vm *);
void string_repeat_in_place(struct string *left, int64_t n);
bool string_is_empty(const struct string *str);
int string_cmp(const struct string *left, const struct string *right);
struct string *string_at(const struct string *str, int64_t n);
array_obj *string_chars(const struct string *str);
size_t string_len(struct string *str);