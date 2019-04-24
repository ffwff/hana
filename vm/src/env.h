#pragma once

#include <stddef.h>
#include "value.h"

struct env {
    struct value *slots;
    size_t nslots;
    struct env *parent;

    uint32_t ifn;
};

void env_init(struct env*, size_t nslots, size_t up_to);
struct value *env_get(struct env*, size_t n);
void env_set(struct env *env, size_t n, struct value *val);
void env_free(struct env*);
