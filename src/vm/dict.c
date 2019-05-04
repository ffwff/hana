#include <string.h>
#include <stdio.h>
#include <assert.h>
#include "dict.h"

void dict_init(struct dict *dict) {
    hmap_init(&dict->data);
    dict->prototypev = NULL;
}
void dict_free(struct dict *dict, __attribute__((unused)) void *p)
{
    hmap_free(&dict->data);
}

struct value *dict_get(struct dict *dict, const char *key) {
    struct value *local = hmap_get(&dict->data, key);
    if(local != NULL) return local;
    if(strcmp(key, "constructor") == 0) return NULL; // don't get parent's constructor
    if(dict->prototypev != NULL) return dict_get(dict->prototypev, key);
    return NULL;
}

struct value *dict_get_hash(struct dict *dict, const char *key, const uint32_t hash) {
    struct value *local = hmap_get_hash(&dict->data, key, hash);
    if(local != NULL) return local;
    if(dict->prototypev != NULL) return dict_get_hash(dict->prototypev, key, hash);
    return NULL;
}

void dict_set(struct dict *dict, const char *key, struct value *val) {
    if(val->type == TYPE_NIL) {
        hmap_del(&dict->data, key);
        if(strcmp(key, "prototype") == 0)
            dict->prototypev = NULL;
    } else {
        struct value *dval = _hmap_set(&dict->data, key, val, 0);
        if(strcmp(key, "prototype") == 0 && val->type == TYPE_DICT)
            dict->prototypev = dval->as.dict;
    }
}

void dict_copy(struct dict *dst, const struct dict *src) {
    hmap_copy(&dst->data, &src->data);
}
