#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "map.h"

void map_init(struct map *map) {
    map->data = calloc(1, sizeof(struct map_entry));
    map->length = 0;
    map->capacity = 1;
}

void map_free(struct map *map) {
    for(size_t i = 0; i < map->length; i++) {
        free(map->data[i]->key);
        value_free(&map->data[i]->val);
        free(map->data[i]);
    }
    free(map->data);
    map->data = 0;
    map->length = 0;
    map->capacity = 0;
}

struct value *map_get(struct map *map, const char *key) {
    for(size_t i = 0; i < map->length; i++)
        if(strcmp(map->data[i]->key, key) == 0)
            return &map->data[i]->val;
    return NULL;
}

struct value *map_set(struct map *map, const char *key, struct value *val) {
    for(size_t i = 0; i < map->length; i++)
        if(strcmp(map->data[i]->key, key) == 0) {
            value_free(&map->data[i]->val);
            value_copy(&map->data[i]->val, val);
            return &map->data[i]->val;
        }
    if(map->length == map->capacity) {
        map->capacity *= 2;
        map->data = realloc(map->data, sizeof(*map->data)*map->capacity);
    }
    map->data[map->length] = malloc(sizeof(struct map_entry));
    map->data[map->length]->key = strdup(key);
    value_copy(&map->data[map->length]->val, val);
    map->length++;
    return &map->data[map->length-1]->val;
}

void map_del(struct map *map, const char *key) {
    for(size_t i = 0; i < map->length; i++)
        if(strcmp(map->data[i]->key, key) == 0) {
            // free entry
            free(map->data[i]->key);
            value_free(&map->data[i]->val);
            free(map->data[i]);
            // move it
            i++;
            for(; i < map->length; i++)
                map->data[i-1] = map->data[i];
            map->length--;
            return;
        }
}

void map_print(struct map *map) {
    for(size_t i = 0; i < map->length; i++) {
        printf("%s: ", map->data[i]->key);
        value_print(&map->data[i]->val);
        printf("\n");
    }
}
