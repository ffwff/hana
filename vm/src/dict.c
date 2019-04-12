#include "dict.h"

void dict_init(struct dict *dict) {
    dict->refs = 1;
    map_init(&dict->data);
}
void dict_free(struct dict *dict) {
    dict->refs--;
    if(dict->refs == 0) {
        map_free(&dict->data);
    }
}

struct value *dict_get(struct dict *dict, const char *key) {
    return map_get(&dict->data, key); }
void dict_set(struct dict *dict, const char *key, struct value *val) {
    map_set(&dict->data, key, val); }
