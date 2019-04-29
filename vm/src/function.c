#include "function.h"
#include <stdlib.h>

void function_init(struct function *fn, uint32_t ip, uint16_t nargs, struct env *parent) {
    fn->refs = 1;
    fn->ip = ip;
    fn->nargs = nargs;
    if(parent != NULL) {
        env_copy(&fn->bound, parent);
        fn->bound.is_function_bound = 1;
    } else {
        fn->bound.is_function_bound = 0;
    }
}

void function_free(struct function *fn) {
    printf("BOUND: 0x%lx\n", fn->bound); // 0x0?
    if(fn->bound.is_function_bound) {
        env_free(&fn->bound);
    }
}
