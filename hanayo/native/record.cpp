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
    const Value val = _arg(vm, TYPE_DICT);
    Value retval; value_array_n(retval, val.v.as.dict->data.keys.length);
    for(size_t i = 0; i < val.v.as.dict->data.keys.length; i++)
        value_str(&retval.v.as.array->data.data[i], val.v.as.dict->data.keys.data[i]);
    _push(vm, retval);
}

fn(is_record) {
    assert(nargs == 1);
    const Value val = _pop(vm);
    Value retval; value_int(retval, val.v.type == TYPE_DICT);
    _push(vm, retval);
}

fn(copy) {
    assert(nargs == 1);
    struct value val = _arg(vm, TYPE_DICT);
    struct value nval;
    value_dict_copy_noref(&nval, val.as.dict);
    array_push(vm->stack, nval);
}
