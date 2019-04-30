#include <stdio.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::integer::name(struct vm *vm, int nargs)

fn(constructor) {
    struct value *val = &array_top(vm->stack);
    if(val->type == TYPE_INT)
        return;
    else if(val->type == TYPE_FLOAT)
        value_int(val, (int64_t)val->as.floatp);
    else if(val->type == TYPE_STR) {
        int64_t i;
        sscanf(string_data(val->as.str), "%ld", &i);
        value_int(val, i);
    } else {
        value_int(val, 0);
    }
}
