#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::float_::name(struct vm *vm, int nargs)

fn(constructor) {
    struct value *val = &array_top(vm->stack);
    if(val->type == TYPE_FLOAT)
        return;
    else if(val->type == TYPE_INT)
        value_float(val, (double)val->as.integer);
    else if(val->type == TYPE_STR) {
        double f;
        sscanf(string_data(val->as.str), "%lf", &f);
        value_float(val, f);
    } else {
        value_float(val, 0);
    }
}
fn(round) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    value_int(val, (int64_t)::round(val->as.floatp));
}
