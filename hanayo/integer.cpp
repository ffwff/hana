#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::integer::name(struct vm *vm, int nargs)

fn(constructor) {
    struct value *val = &array_top(vm->stack);
    if(val->type == value::TYPE_INT)
        return;
    else if(val->type == value::TYPE_FLOAT)
        value_int(val, (int64_t)val->as.floatp);
    else if(val->type == value::TYPE_STR) {
        value_free(val);
        value_int(val, std::stoi(string_data(val->as.str)));
    } else {
        value_free(val);
        value_int(val, 0);
    }
}
