#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "value.h"

struct map_entry {
    char *key;
    struct value val;
};

struct map {
    struct map_entry *data;
    size_t length, capacity;
};

void map_init(struct map *);
void map_free(struct map *);
struct value *map_get(struct map *, const char *);
struct value *map_set(struct map *, const char *, struct value *);
void map_del(struct map *, const char *);
void map_print(struct map *);

#ifdef __cplusplus
}
#endif
