#include "function.h"
#include <stdlib.h>

void function_init(struct function *fn, uint32_t ip, uint16_t nargs, struct env *parent) {
    fn->refs = 1;
    fn->ip = ip;
    fn->nargs = nargs;
    if(parent) {
        fn->bound = parent;
        fn->bound->is_function_bound = 1;
    } else {
        fn->bound = NULL;
    }
}

void function_free(struct function *fn) {
    if(fn->bound != NULL) {
        env_free(fn->bound);
        free(fn->bound);
    }
}
