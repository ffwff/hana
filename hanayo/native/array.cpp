#include <assert.h>
#include "hanayo.h"
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"

#define fn_(name) void hanayo::array::name(struct vm *vm, int nargs)

fn_(constructor) {
    Value aval;
    if(nargs == 0)
        value_array(aval);
    else {
        value_array_n(aval, nargs);
        aval.v.as.array->data.length = nargs;
        for(size_t i = 0; i < (size_t)nargs; i++) {
            Value val = _pop(vm);
            value_copy(&aval.v.as.array->data.data[i], val);
        }
    }
    _push(vm, aval);
}

fn_(length) {
    assert(nargs == 1);
    const Value val = _arg(vm, value::value_type::TYPE_ARRAY);
    Value retval; value_int(retval, val.v.as.array->data.length);
    _push(vm, retval);
}
fn_(delete_) {
    assert(nargs == 3);

    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    const size_t from_pos = (size_t)_arg(vm, value::TYPE_INT).v.as.integer;
    const size_t nelems = (size_t)_arg(vm, value::TYPE_INT).v.as.integer;

    assert(from_pos >= 0 && from_pos+nelems < aval.v.as.array->data.length);
    for(size_t i = from_pos; i < (from_pos+nelems); i++) {
        value_free(&aval.v.as.array->data.data[i]);
    }
    size_t remaining = (aval.v.as.array->data.length-from_pos-nelems)*sizeof(struct value);
    memmove(&aval.v.as.array->data.data[from_pos],
            &aval.v.as.array->data.data[from_pos+nelems], remaining);
    aval.v.as.array->data.length -= nelems;
    _push(vm, aval);
}
fn_(copy) {
    if(nargs == 1) {
        const Value val = _arg(vm, value::value_type::TYPE_ARRAY);

        Value aval; value_array_n(aval, val.v.as.array->data.length);
        for(size_t i = 0; i < val.v.as.array->data.length; i++)
            value_copy(&aval.v.as.array->data.data[i], &val.v.as.array->data.data[i]);

        _push(vm, aval);
        return;
    }

    assert(nargs == 3);

    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    const int64_t from_pos = _arg(vm, value::TYPE_INT).v.as.integer;
    const int64_t nelems = _arg(vm, value::TYPE_INT).v.as.integer;

    if(nelems == 0) {
        _push(vm, aval);
    } else {
        Value newval; value_array_n(newval, nelems);
        for(int64_t i = from_pos, j = 0; i < (from_pos+nelems); i++, j++) {
            value_copy(&newval.v.as.array->data.data[j], &aval.v.as.array->data.data[i]);
        }
        newval.v.as.array->data.length = nelems;
        _push(vm, newval);
    }
}
fn_(at) {
    assert(nargs == 2);

    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    const int64_t index = _arg(vm, value::TYPE_INT).v.as.integer;

    if(index < (int64_t)(aval.v.as.array->data.length) && index >= 0) {
        Value val = Value::copy(aval.v.as.array->data.data[index]);
        _push(vm, val);
    } else {
        Value val;
        _push(vm, val);
    }
}
fn_(index) {
    assert(nargs == 2);

    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    Value needle = _pop(vm);

    for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
        struct value result;
        value_eq(&result, needle, &aval.v.as.array->data.data[i]);
        if(result.as.integer == 1) {
            Value retval; value_int(retval, (int64_t)i);
            _push(vm, retval);
            return;
        }
    }
    Value retval; value_int(retval, 0);
    _push(vm, retval);
}
fn_(insert) {
    assert(nargs == 3);

    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    const int64_t from_pos = _arg(vm, value::TYPE_INT).v.as.integer;
    Value newval = _pop(vm);

    if(from_pos < 0) {
        _push(vm, aval);
        return;
    } else if(from_pos >= (int64_t)aval.v.as.array->data.length) {
        for(int64_t i = aval.v.as.array->data.length; i < from_pos+1; i++)
            array_push(aval.v.as.array->data, {0});
    } else {
        array_push(aval.v.as.array->data, {0}); // reserve space
    }
    size_t n = (aval.v.as.array->data.length-from_pos)*sizeof(struct value);
    memmove(&aval.v.as.array->data.data[from_pos+1],
            &aval.v.as.array->data.data[from_pos],
            n
    );
    aval.v.as.array->data.data[from_pos] = newval.deref();
    _push(vm, aval);

}
fn_(push) {
    assert(nargs == 2);

    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    Value newval = _pop(vm);
    array_push(aval.v.as.array->data, newval.deref());
    _push(vm, aval);
}
fn_(pop) {
    assert(nargs == 1);
    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    if(aval.v.as.array->data.length == 0) {
        Value retval; _push(vm, retval);
        return;
    }
    Value retval = Value::move(aval.v.as.array->data.data[aval.v.as.array->data.length-1]);
    aval.v.as.array->data.length--;
    _push(vm, retval);
}

#define SORT_ARRAY(x) \
qsort(x->data.data, x->data.length, sizeof(struct value), [](const void *a, const void *b) -> int { \
    const struct value *va = (const struct value *)a, \
                       *vb = (const struct value *)b; \
    struct value ret = {0}; \
    value_lt(&ret, va, vb); \
    if(ret.as.integer == 1) return -1; \
    value_eq(&ret, va, vb); \
    if(ret.as.integer == 1) return 0; \
    return 1; \
});
fn_(sort) {
    assert(nargs == 1);
    Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    Value retval; value_array_n(retval, aval.v.as.array->data.length);
    for(size_t i = 0; i < aval.v.as.array->data.length; i++)
        value_copy(&retval.v.as.array->data.data[i], &aval.v.as.array->data.data[i]);
    retval.v.as.array->data.length = aval.v.as.array->data.length;
    SORT_ARRAY(retval.v.as.array)
    _push(vm, retval);
}
fn_(sort_) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    SORT_ARRAY(val->as.array)
}

fn_(map) {
    assert(nargs == 2);

    const Value aval = _arg(vm, value::value_type::TYPE_ARRAY);

    // cb (should be function)
    Value fn = _pop(vm);
    assert(fn.v.type == value::TYPE_FN || // TODO move this somewhere else
        fn.v.type == value::TYPE_DICT  ||
        fn.v.type == value::TYPE_NATIVE_FN);

    if(aval.v.as.array->data.length == 0) {
        Value retval; _push(vm, retval);
        return;
    }

    // init array with same length
    Value newval; value_array_n(newval, aval.v.as.array->data.length);

    if(fn.v.type != value::TYPE_NATIVE_FN) {
        for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
            // setup arguments
            a_arguments args = {
                .data = (struct value*)calloc(1, sizeof(struct value)),
                .length = 1,
                .capacity = 1
            };
            value_copy(&args.data[0], &aval.v.as.array->data.data[i]);
            // return
            struct value *ret = vm_call(vm, &fn.v, args);
            if(ret == (struct value*)-1) {
                value_free(&args.data[0]);
                array_free(args);
                return;
            }
            value_copy(&newval.v.as.array->data.data[i], ret);
            // cleanup
            value_free(&args.data[0]);
            array_free(args);
            value_free(ret);
            array_pop(vm->stack);
        }
    } else {
        for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
            // arguments
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &aval.v.as.array->data.data[i]);
            // call function
            fn.v.as.fn(vm, 1);
            newval.v.as.array->data.data[i] = array_top(vm->stack);
            // cleanup
            value_free(&array_top(vm->stack));
            array_pop(vm->stack);
        }
    }

    _push(vm, newval);

}
fn_(filter) {
    assert(nargs == 2);

    const Value aval = _arg(vm, value::value_type::TYPE_ARRAY);

    // cb (should be function)
    Value fn = _pop(vm);
    assert(fn.v.type == value::TYPE_FN ||
        fn.v.type == value::TYPE_DICT  ||
        fn.v.type == value::TYPE_NATIVE_FN);

    if(aval.v.as.array->data.length == 0) {
        Value retval; _push(vm, retval);
        return;
    }

    // init array with same length
    Value newval; value_array(newval);

    if(fn.v.type != value::TYPE_NATIVE_FN) {
        for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
            // setup arguments
            a_arguments args = {
                .data = (struct value*)calloc(1, sizeof(struct value)),
                .length = 1,
                .capacity = 1
            };
            value_copy(&args.data[0], &aval.v.as.array->data.data[i]);
            // return
            struct value *ret = vm_call(vm, fn, args);
            if(ret == (struct value*)-1) {
                value_free(&args.data[0]);
                array_free(args);
                return;
            }
            if(value_is_true(ret)) {
                struct value val;
                value_copy(&val, &aval.v.as.array->data.data[i]);
                array_push(newval.v.as.array->data, val);
            }
            // cleanup
            value_free(&args.data[0]);
            array_free(args);
            value_free(ret);
            array_pop(vm->stack);
        }
    } else {
        for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
            // arguments
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &aval.v.as.array->data.data[i]);
            // call function
            fn.v.as.fn(vm, 1);
            if(value_is_true(&array_top(vm->stack))) {
                struct value val;
                value_copy(&val, &aval.v.as.array->data.data[i]);
                array_push(newval.v.as.array->data, val);
            }
            // cleanup
            value_free(&array_top(vm->stack));
            array_pop(vm->stack);
        }
    }

    _push(vm, newval);
}
fn_(reduce) {
    assert(nargs == 3);

    const Value aval = _arg(vm, value::value_type::TYPE_ARRAY);

    // cb (should be function)
    Value fn = _pop(vm);
    assert(fn.v.type == value::TYPE_FN ||
        fn.v.type == value::TYPE_DICT  ||
        fn.v.type == value::TYPE_NATIVE_FN);

    if(aval.v.as.array->data.length == 0) {
        Value retval; _push(vm, retval);
        return;
    }

    // accumulator
    Value acc = _pop(vm);

    if(fn.v.type != value::TYPE_NATIVE_FN) {
        for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
            // setup arguments
            a_arguments args = {
                .data = (struct value*)calloc(2, sizeof(struct value)),
                .length = 2,
                .capacity = 2
            };
            value_copy(&args.data[0], &acc.v);
            value_copy(&args.data[1], &aval.v.as.array->data.data[i]);
            // return
            struct value *ret = vm_call(vm, fn, args);
            if(ret == (struct value*)-1) {
                value_free(&args.data[0]);
                value_free(&args.data[1]);
                array_free(args);
                return;
            }
            Value newval = _pop(vm);
            Value::move(acc, newval);
            // cleanup
            value_free(&args.data[0]);
            value_free(&args.data[1]);
            array_free(args);
        }
    } else {
        for(size_t i = 0; i < aval.v.as.array->data.length; i++) {
            // arguments
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &aval.v.as.array->data.data[i]);
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &acc.v);
            // call function
            fn.v.as.fn(vm, 2);
            Value newval = _pop(vm);
            Value::move(acc, newval);
        }
    }

    _push(vm, acc);
}

fn_(join) {
    assert(nargs == 2);

    // array
    const Value aval = _arg(vm, value::value_type::TYPE_ARRAY);
    const Value jval = _arg(vm, value::value_type::TYPE_STR);
    const auto joiner = string_data(jval.v.as.str);
    const auto joiner_len = string_len(jval.v.as.str);

    size_t len = 0;
    char *s = (char*)malloc(len+1);
    s[0] = 0;
    if(aval.v.as.array->data.length) {
        assert(aval.v.as.array->data.data[0].type == value::TYPE_STR);
        const auto ss = aval.v.as.array->data.data[0].as.str;
        len += string_len(ss); s = (char*)realloc(s, len+1);
        strcat(s, string_data(ss));
    }
    for(size_t i = 1; i < aval.v.as.array->data.length; i++) {
        assert(aval.v.as.array->data.data[i].type == value::TYPE_STR);
        const auto ss = aval.v.as.array->data.data[i].as.str;
        len += joiner_len + string_len(ss); s = (char*)realloc(s, len+1);
        strcat(s, joiner);
        strcat(s, string_data(ss));
    }
    s[len] = 0;

    Value retval;
    value_str(retval, s); free(s);
    _push(vm, retval);
}
