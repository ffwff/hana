#include <iostream>
#include "hanayo.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

fn(print) {
    int written = 0;
    while(nargs--) {
        struct value val = array_top(vm->stack);
        const auto s = hanayo::_to_string(val);
        written += fputs(s.data(), stdout);
        value_free(&val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
    array_push(vm->stack, val);
}

fn(input) {
    std::string s;
    std::cin >> s;
    struct value val;
    value_str(&val, s.data());
    array_push(vm->stack, val);
}
