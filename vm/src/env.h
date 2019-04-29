#pragma once

#include <stddef.h>
#include <stdbool.h>
#include "value.h"

struct env { // function's stack frame
    struct value *slots;
    size_t nslots;
    struct env *parent;

    bool is_function_bound;
    struct env *lexical_parent;
    // lexical parents are the parent of the function's lexical scopes
    // this should be set to (struct function*)->bound

    uint32_t retip;
};

void env_init(struct env*, size_t nslots);
void env_copy(struct env *dst, struct env *src);
struct value *env_get(struct env*, size_t n);
void env_set(struct env *env, size_t n, struct value *val);
void env_free(struct env*);
