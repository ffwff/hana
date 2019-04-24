#include <stdlib.h>
#include <string.h>
#include "env.h"

#define is_slim(n) (n<=64)
#define bit_check(var,pos) ((var >> pos) & 1U)
#define bit_set(var,pos) var |= 1UL << pos

void env_init(struct env *env, size_t nslots, size_t up_to) {
    env->slots = calloc(nslots, sizeof(struct value));
    env->nslots = nslots;
    if(env->parent != NULL)
        for(size_t i = 0; i < up_to; i++)
            value_copy(&env->slots[i], &env->parent->slots[i]);
    // NOTE: parent and caller is already init in OP_CALL
}

struct value *env_get(struct env *env, size_t n) {
    return &env->slots[n];
}

void env_set(struct env *env, size_t n, struct value *val) {
    value_free(&env->slots[n]);
    value_copy(&env->slots[n], val);
}

void env_free(struct env *env) {
    if(env->slots == NULL) return;
    for(int i = 0; i < env->nslots; i++)
    { value_free(&env->slots[i]); }
    free(env->slots);
}
