#pragma once
#include "value.h"

struct map_entry {
    char *key;
    struct value val;
};

struct map {
    struct map_entry **data;
    size_t length, capacity;
};

void map_init(struct map *);
void map_free(struct map *);
struct value *map_get(struct map *, char *);
void map_set(struct map *, char *, struct value *);
void map_del(struct map *, char *);
void map_print(struct map *);
