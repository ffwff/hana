#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "map.h"
struct dict {
    struct map data;
    size_t refs;
};

void dict_init(struct dict*);
void dict_free(struct dict*);
struct value *dict_get(struct dict *dict, const char *key);
void dict_set(struct dict *dict, const char *key, struct value *val);

#ifdef __cplusplus
}
#endif
