#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "value.h"
#include "array.h"

struct hmap;
struct string;

struct hmap *hmap_malloc();
void hmap_free(struct hmap *);
const struct value *hmap_get(struct hmap *, const char *);
void hmap_set(struct hmap *, const char *, struct value *);
const struct value *hmap_get_str(struct hmap *, struct string *);
void hmap_set_str(struct hmap *, struct string *, struct value *);

#ifdef __cplusplus
}
#endif
