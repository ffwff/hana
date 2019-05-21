#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "value.h"
#include "array.h"

struct dict;
struct string;

struct dict *dict_malloc();
const struct value *dict_get(const struct dict *, const char *);
void dict_set(struct dict *, const char *, struct value);
const struct value *dict_get_str(const struct dict *, struct string *);
void dict_set_str(struct dict *, struct string *, struct value);

#ifdef __cplusplus
}
#endif
