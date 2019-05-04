#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "value.h"
#include "array.h"

struct hmap {
    // FILLER
};

void hmap_init(struct hmap *);
void hmap_free(struct hmap *);
struct value *hmap_get(struct hmap *, const char *);
struct value *hmap_set(struct hmap *, const char *, struct value *);

#ifdef __cplusplus
}
#endif
