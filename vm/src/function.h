#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "env.h"

struct function {
    uint32_t refs;
    uint32_t ip;
    uint16_t nargs;
    struct env bound;
    // NOTE: bound represents the current local environment
    // at the time the function is declared, this will be
    // COPIED into another struct env whenever OP_CALL is issued
    // We use this to implement closures
};

void function_init(struct function *, uint32_t addr, uint16_t nargs, struct env *env);
void function_free(struct function *);

#ifdef __cplusplus
}
#endif
