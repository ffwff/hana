#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>

struct env;
struct function {
    uint32_t ip;
    uint16_t nargs;
    // ... (additional rust properties)
};

struct function *function_malloc(uint32_t addr, uint16_t nargs, struct env *env);
void function_set_bound_var(struct function *, uint16_t n, struct value val);

#ifdef __cplusplus
}
#endif
