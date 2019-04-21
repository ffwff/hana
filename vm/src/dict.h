#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "hmap.h"
struct dict {
    size_t refs;
    struct hmap data;
    struct dict *prototypev;
};

void dict_init(struct dict*);
void dict_free(struct dict*);
struct value *dict_get(struct dict *dict, const char *key);
struct value *dict_get_hash(struct dict *dict, const char *key, const uint32_t hash);
void dict_set(struct dict *dict, const char *key, struct value *val);
void dict_copy(struct dict *dst, struct dict *src);

#ifdef __cplusplus
}
#endif
