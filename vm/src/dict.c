#include <string.h>
#include "dict.h"

void dict_init(struct dict *dict) {
    dict->refs = 1;
    map_init(&dict->data);
    dict->prototypev = NULL;
}
void dict_free(struct dict *dict) {
    dict->refs--;
    if(dict->refs == 0)
        map_free(&dict->data);
}

struct value *dict_get(struct dict *dict, const char *key) {
    struct value *local = map_get(&dict->data, key);
    if(local != NULL) return local;
    if(dict->prototypev != NULL && dict->prototypev->type == TYPE_DICT)
        return dict_get(dict->prototypev->as.dict, key);
    return NULL;
}

void dict_set(struct dict *dict, const char *key, struct value *val) {
    struct value *dval = map_set(&dict->data, key, val);
    if(strcmp(key, "prototype") == 0)
        dict->prototypev = dval;
}

void dict_copy(struct dict *dst, struct dict *src) {
    /*for(size_t i = 0; i < src->data.length; i++) {
        struct map_entry *entry = src->data.data[i];
        map_set(&dst->data, entry->key, &entry->val);
    }*/

}
