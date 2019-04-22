#include <stdlib.h>
#include "env.h"

void env_init(struct env *env, size_t nslots) {
    env->slots = calloc(nslots, sizeof(struct value));
    env->popslots = bit_array_create(nslots);
    env->nslots = nslots;
    env->parent = NULL;
}

struct value *env_get(struct env *env, size_t n) {
    if(bit_array_get_bit(env->popslots, n)) {
        return &env->slots[n];
    } else if(env->parent != NULL) {
        struct value *val = env_get(env->parent, n);
        if(val != NULL) {
            bit_array_set_bit(env->popslots,n);
            value_copy(&env->slots[n], val);
            return &env->slots[n];
        }
        return NULL;
    }
    return NULL;
}

void env_set(struct env *env, size_t n, struct value *val) {
    value_free(&env->slots[n]);
    value_copy(&env->slots[n], val);
    bit_array_set_bit(env->popslots, n);
}

void env_inherit(struct env *dst, struct env *src, size_t nslots) {
    env_init(dst, nslots);
    dst->parent = src;
}

void env_free(struct env *env) {
    for(int i = 0; i < env->nslots; i++)
        value_free(&env->slots[i]);
    bit_array_free(env->popslots);
    free(env->slots);
}
