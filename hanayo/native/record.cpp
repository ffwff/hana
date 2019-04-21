#include "hanayo.h"
#include "vm/src/dict.h"
#include "vm/src/array_obj.h"

#define fn(name) void hanayo::record::name(struct vm *vm, int nargs)

fn(constructor) {
    assert(nargs == 0);
    struct value val; value_dict(&val);
    array_push(vm->stack, val);
}

fn(keys) {
    assert(nargs == 1);
    struct value val = _arg(vm, value::TYPE_DICT);
    struct value aval;
    value_array_n(&aval, val.as.dict->data.keys.length);
    for(size_t i = 0; i < val.as.dict->data.keys.length; i++) {
        struct value sval;
        value_str(&sval, val.as.dict->data.keys.data[i]);
        aval.as.array->data.data[i] = sval;
    }
    value_free(&val);
    array_push(vm->stack, aval);
}
