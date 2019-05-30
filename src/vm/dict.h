#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "value.h"
#include "array.h"

struct dict;
struct string;
struct vm;

struct dict *dict_malloc(const struct vm *vm);
struct dict *dict_malloc_n(const struct vm *vm, size_t n);
const struct value *dict_get(const struct dict *, const char *);
void dict_set(struct dict *, const char *, struct value);
const struct value *dict_get_str(const struct dict *, struct string *);
void dict_set_str(struct dict *, struct string *, struct value);

#ifdef __cplusplus
}
#endif
