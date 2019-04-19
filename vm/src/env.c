#include <stdlib.h>
#include "env.h"

void env_init(struct env *env, size_t nslots) {
    env->slots = malloc(sizeof(struct value)*nslots);
    env->nslots = nslots;
    env->parent = NULL;
}

void env_inherit(struct env *dst, struct env *src, size_t nslots) {
    env_init(dst, nslots);
    dst->parent = src;
    for(size_t i = 0; i < nslots; i++)
        value_copy(&dst->slots[i], &src->slots[i]);
}

void env_free(struct env *env) {
    for(size_t i = 0; i < env->nslots; i++)
        value_free(&env->slots[i]);
    free(env->slots);
}
