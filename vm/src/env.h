#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include "hmap.h"

struct env {
    struct hmap data;
    struct env *parent;
};
void env_init(struct env *, struct env *);
void env_free(struct env *);
struct value *env_get(struct env *env, const char *key);
struct value *env_get_hash(struct env *env, const char *key, const uint64_t hash);
void env_set(struct env *env, const char *key, struct value *val);
void env_del(struct env *env, const char *key);

#ifdef __cplusplus
}
#endif
