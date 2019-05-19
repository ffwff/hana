#pragma once

#include <stddef.h>
#include <stdbool.h>
#include "value.h"

struct env;
void env_init(struct env *, uint16_t nslots, struct vm *vm);
struct env *env_copy(struct env *src);
struct value env_get(struct env *, uint16_t n);
struct value env_get_up(struct env *, uint16_t up, uint16_t n);
void env_set(struct env *env, uint16_t n, struct value val);
