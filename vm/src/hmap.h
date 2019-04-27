#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "value.h"
#include "array.h"

struct hmap_entry {
    char *key;
    struct value val;
};
typedef array(struct hmap_entry) a_hmap_entry;
typedef array(a_hmap_entry) a_hmap_buckets;
typedef array(char*) a_hmap_keys;

struct hmap {
    a_hmap_buckets data;
    size_t occupied;
    a_hmap_keys keys;
};

void hmap_init(struct hmap *);
void hmap_free(struct hmap *);
struct value *hmap_get(struct hmap *, const char *);
struct value *hmap_get_hash(struct hmap *, const char *, const uint32_t);
struct value *_hmap_set(struct hmap *, const char *, struct value *, int);
#define hmap_set(hmap, key, val) _hmap_set(hmap, key, val, 0)
void hmap_copy(struct hmap *, const struct hmap *);
void hmap_del(struct hmap *, const char *);
void hmap_print(struct hmap *);

#ifdef __cplusplus
}
#endif
