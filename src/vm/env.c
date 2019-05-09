#include <stdlib.h>
#include <string.h>
#include "string_.h"
#include "env.h"

void env_init(struct env *env, size_t nslots) {
    if(env->slots == NULL) {
        env->slots = calloc(nslots, sizeof(struct value));
        env->nslots = nslots;
    } else if(env->nslots < nslots) {
        free(env->slots);
        env->slots = calloc(nslots, sizeof(struct value));
        env->nslots = nslots;
    } else { // reused env for tail call
        memset(env->slots, 0, env->nslots*sizeof(struct value));
        env->nslots = nslots;
    }
    // NOTE: parent and caller is already init in OP_CALL
}

void env_copy(struct env *dst, struct env *src) {
    dst->nslots = src->nslots;
    dst->slots = calloc(src->nslots, sizeof(struct value));
    for(int i = 0; i < src->nslots; i++)
        value_copy(&dst->slots[i], &src->slots[i]);
}

struct value *env_get(struct env *env, size_t n) {
    return &env->slots[n];
}

void env_set(struct env *env, size_t n, struct value *val) {
    value_copy(&env->slots[n], val);
}

void env_free(struct env *env) {
    if(env->slots == NULL) return;
    free(env->slots);
}
