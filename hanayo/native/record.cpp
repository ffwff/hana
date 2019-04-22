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
        value_str(&aval.as.array->data.data[i], val.as.dict->data.keys.data[i]);
    }
    value_free(&val);
    array_push(vm->stack, aval);
}

fn(is_record) {
    assert(nargs == 1);
    struct value val = array_top(vm->stack);
    int is_record = val.type == value::TYPE_DICT;
    array_pop(vm->stack);
    value_free(&val);
    struct value ret;
    value_int(&ret, is_record);
    array_push(vm->stack, ret);
}
