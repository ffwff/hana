#include <string.h>
#include "dict.h"

void dict_init(struct dict *dict) {
    dict->refs = 1;
    hmap_init(&dict->data);
    dict->prototypev = NULL;
}
void dict_free(struct dict *dict) {
    dict->refs--;
    if(dict->refs == 0)
        hmap_free(&dict->data);
}

struct value *dict_get(struct dict *dict, const char *key) {
    struct value *local = hmap_get(&dict->data, key);
    if(local != NULL) return local;
    if(dict->prototypev != NULL && dict->prototypev->type == TYPE_DICT)
        return dict_get(dict->prototypev->as.dict, key);
    return NULL;
}

struct value *dict_get_hash(struct dict *dict, const char *key, const uint32_t hash) {
    struct value *local = hmap_get_hash(&dict->data, key, hash);
    if(local != NULL) return local;
    if(dict->prototypev != NULL && dict->prototypev->type == TYPE_DICT)
        return dict_get_hash(dict->prototypev->as.dict, key, hash);
    return NULL;
}

void dict_set(struct dict *dict, const char *key, struct value *val) {
    int has_grown = 0;
    struct value *dval = _hmap_set(&dict->data, key, val, 0, &has_grown);
    if(strcmp(key, "prototype") == 0) {
        dict->prototypev = dval;
    } else if(has_grown) {
        dict->prototypev = hmap_get(&dict->data, "prototype");
    }
}

void dict_copy(struct dict *dst, struct dict *src) {
    /*for(size_t i = 0; i < src->data.length; i++) {
        struct hmap_entry *entry = src->data.data[i];
        hmap_set(&dst->data, entry->key, &entry->val);
    }*/

}
