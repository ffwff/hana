#include <stdlib.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

fn(getenv) {
    assert(nargs == 1);
    auto val = _arg(vm, value::TYPE_STR);
    auto env = ::getenv(string_data(val.as.str));
    if(env == NULL) array_push(vm->stack, {0});
    else {
        struct value val;
        value_str(&val, env);
        array_push(vm->stack, val);
    }
    value_free(&val);
}

fn(setenv) {
    assert(nargs == 2);
    auto kval = _arg(vm, value::TYPE_STR);
    auto vval = _arg(vm, value::TYPE_STR);
    struct value val;
    value_int(&val, ::setenv(string_data(kval.as.str), string_data(vval.as.str), 1));
    array_push(vm->stack, val);
    value_free(&kval); value_free(&vval);
}
