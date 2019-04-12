#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include "map.h"

struct env {
    struct map data;
    struct env *parent;
};
void env_init(struct env *, struct env *);
void env_free(struct env *);
struct value *env_get(struct env *env, const char *key);
void env_set(struct env *env, const char *key, struct value *val);
void env_set_local(struct env *env, const char *key, struct value *val);
void env_del(struct env *env, const char *key);

#ifdef __cplusplus
}
#endif
