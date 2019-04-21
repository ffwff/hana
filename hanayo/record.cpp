#include "hanayo.h"

#define fn(name) void hanayo::record::name(struct vm *vm, int nargs)

fn(constructor) {
    assert(nargs == 0);
    struct value val; value_dict(&val);
    array_push(vm->stack, val);
}
