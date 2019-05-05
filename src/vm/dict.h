#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>

struct dict;
void dict_init(struct dict*);
void dict_free(struct dict*, void*);
struct value *dict_get(struct dict *dict, const char *key);
struct dict *dict_get_prototype(struct dict*);
void dict_set(struct dict *dict, const char *key, struct value *val);

#ifdef __cplusplus
}
#endif
