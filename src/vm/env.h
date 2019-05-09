#pragma once

#include <stddef.h>
#include <stdbool.h>
#include "value.h"

struct env;
struct env *env_malloc(struct env *old, uint32_t retip, struct env *lexical_parent, uint16_t nargs);
void env_init(struct env *, uint16_t nslots, struct vm *vm);
struct env *env_copy(struct env *src);
struct value env_get(struct env *, uint16_t n);
struct value env_get_up(struct env *, uint16_t up, uint16_t n);
void env_set(struct env *env, uint16_t n, struct value val);
void env_set_up(struct env *, uint16_t up, uint16_t n, struct value val);