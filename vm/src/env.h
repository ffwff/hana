#pragma once

#include <stddef.h>
#include "value.h"

struct env {
    struct value *slots;
    size_t nslots;
    struct env *parent;
};

void env_init(struct env*, size_t nslots);
void env_inherit(struct env*, struct env*, size_t nslots);
void env_free(struct env*);
