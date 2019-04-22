#include <stdlib.h>
#include <math.h>
#include "env.h"

#define is_slim(n) (n<=64)
#define bit_check(var,pos) ((var >> pos) & 1U)
#define bit_set(var,pos) var |= 1UL << pos

void env_init(struct env *env, size_t nslots) {
    env->slots = malloc(sizeof(struct value)*nslots);
    // NOTE: this assumes that nslots <= 64
    env->popslot = 0;
    env->nslots = nslots;
    // NOTE: parent and caller is already init in OP_CALL
}

struct value *env_get(struct env *env, size_t n) {
    if(bit_check(env->popslot, n)) {
        return &env->slots[n];
    } else if(env->parent != NULL) {
        value_copy(&env->slots[n], env_get(env->parent, n));
        bit_set(env->popslot, n);
        return &env->slots[n];
    }
    return NULL;
}

void env_set(struct env *env, size_t n, struct value *val) {
    value_free(&env->slots[n]);
    value_copy(&env->slots[n], val);
    bit_set(env->popslot, n);
}

void env_free(struct env *env) {
    return;
    if(env->slots == NULL) return;
    for(int i = 0; i < env->nslots; i++)
        value_free(&env->slots[i]);
    free(env->slots);
}
