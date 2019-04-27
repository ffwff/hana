#include <stdio.h>
#include <limits.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

fn(print) {
    int written = 0;
    while(nargs--) {
        struct value val = array_top(vm->stack);
        const auto s = hanayo::_to_string(val);
        written += fputs(s, stdout);
        free(s);
        value_free(&val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
    array_push(vm->stack, val);
}

fn(input) {
    assert(nargs == 1);
    char *line = nullptr;
    size_t n = 0;
    getline(&line, &n, stdin);
    struct value val;
    value_str(&val, line); free(line);
    array_push(vm->stack, val);
}
