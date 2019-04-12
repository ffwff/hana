#include <stddef.h>
#include "env.h"

void env_init(struct env *env, struct env *parent) {
    map_init(&env->data);
    env->parent = parent;
}
void env_free(struct env *env) {
    map_free(&env->data);
}

struct value *env_get(struct env *env, const char *key) {
    struct value *local = map_get(&env->data, key);
    if(local != NULL) return local;
    else if(env->parent != NULL) return env_get(env->parent, key);
    return NULL;
}
void env_set(struct env *env, const char *key, struct value *val) {
    struct value *local = map_get(&env->data, key);
    if(local != NULL) { // set existing local var
        value_free(local);
        value_copy(local, val);
        return;
    } else if (env->parent != NULL) {
        struct value *outer = env_get(env->parent, key);
        if(outer != NULL) { // set outer scope's var
            value_free(local);
            value_copy(local, val);
            return;
        }
    }
    // set new local var
    map_set(&env->data, key, val);
}
void env_set_local(struct env *env, const char *key, struct value *val) {
    map_set(&env->data, key, val);
}
void env_del(struct env *env, const char *key) {
    // delete local var
    struct value *local = map_get(&env->data, key);
    if(local != NULL) map_del(&env->data, key);
}
