#include <stdio.h>
#include <limits.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

fn(print) {
    int written = 0;
    while(nargs--) {
        auto val = _pop(vm);
        const auto s = hanayo::_to_string(val);
        written += fputs(s, stdout);
        free(s);
    }
    {
        Value retval;
        value_int(retval, written);
        _push(vm, retval);
    }
}

fn(input) {
    assert(nargs == 1);
    char *line = nullptr;
    size_t n = 0;
    getline(&line, &n, stdin);
    Value val;
    value_str(val, line);
    free(line);
    array_push(vm->stack, val);
}
