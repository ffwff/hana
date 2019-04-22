#pragma once

#include <stddef.h>
#include "value.h"
#include "bit_array.h"

struct env {
    struct value *slots;
    BIT_ARRAY *popslots;
    size_t nslots;
    struct env *parent;
};

void env_init(struct env*, size_t nslots);
struct value *env_get(struct env*, size_t n);
void env_set(struct env *env, size_t n, struct value *val);
void env_inherit(struct env*, struct env*, size_t nslots);
void env_free(struct env*);
