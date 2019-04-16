#include "hanayo.h"

#define fn(name) void hanayo::record::name(struct vm *vm, int nargs)

fn(constructor) {
    struct value val; value_dict(&val);
    array_push(vm->stack, val);
}
