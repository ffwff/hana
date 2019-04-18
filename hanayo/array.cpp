#include <cassert>
#include "hanayo.h"
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"

#define fn_(name) void hanayo::array::name(struct vm *vm, int nargs)

fn_(constructor) {
    struct value aval;
    if(nargs == 0) value_array(&aval);
    else {
        value_array_n(&aval, nargs);
        aval.as.array->data.length = nargs;
        for(size_t i = 0; i < (size_t)nargs; i++) {
            struct value val = array_top(vm->stack);
            value_copy(&aval.as.array->data.data[i], &val);
            array_pop(vm->stack);
            value_free(&val);
        }
    }
    array_push(vm->stack, aval);
}

fn_(length) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    int64_t length = val->as.array->data.length;
    value_free(val);
    value_int(val, length);
}
fn_(delete_) {
    assert(nargs == 3);

    struct value val = {0};

    // array
    val = array_top(vm->stack);
    struct value aval;
    value_copy(&aval, &val);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    array_pop(vm->stack);
    size_t from_pos = (size_t)val.as.integer;

    // n chars
    val = array_top(vm->stack);
    array_pop(vm->stack);
    size_t nchars = (size_t)val.as.integer;

    assert(from_pos >= 0 && from_pos+nchars < aval.as.array->data.length);
    for(size_t i = from_pos; i < (from_pos+nchars); i++) {
        value_free(&aval.as.array->data.data[i]);
    }
    size_t remaining = (aval.as.array->data.length-nchars)*sizeof(struct value);
    memmove(&aval.as.array->data.data[from_pos],
            &aval.as.array->data.data[from_pos+nchars], remaining);
    aval.as.array->data.length -= nchars;
    array_push(vm->stack, aval);
}
fn_(copy) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    val = array_top(vm->stack);
    struct value aval;
    value_copy(&aval, &val);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t from_pos = val.as.integer;

    // n chars
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t nchars = val.as.integer;

    if(nchars == 0) {
        value_free(&aval);
        value_array(&aval);
        array_push(vm->stack, val);
    } else {
        struct value newval;
        value_array_n(&newval, nchars);
        for(size_t i = (size_t)from_pos, j = 0; i < (size_t)(from_pos+nchars); i++, j++) {
            value_copy(&newval.as.array->data.data[j], &aval.as.array->data.data[i]);
        }
        newval.as.array->data.length = nchars;
        value_free(&aval);
        array_push(vm->stack, newval);
    }
}
fn_(at) {
    assert(nargs == 2);
    struct value val = {0};

    // string
    val = array_top(vm->stack);
    struct value aval;
    value_copy(&aval, &val);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t index = val.as.integer;

    if(index < (int64_t)(aval.as.array->data.length) && index >= 0) {
        struct value val;
        value_copy(&val, &aval.as.array->data.data[index]);
        value_free(&aval);
        array_push(vm->stack, val);
    } else {
        value_free(&aval);
        array_push(vm->stack, {0});
    }
}
fn_(index) {
    assert(nargs == 2);
    struct value val = {0};

    // string
    val = array_top(vm->stack);
    struct value aval = val;
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    struct value needle;
    value_copy(&needle, &val);
    value_free(&val);
    array_pop(vm->stack);

    array_push(vm->stack, {0});
    for(size_t i = 0; i < aval.as.array->data.length; i++) {
        struct value result;
        value_eq(&result, &needle, &aval.as.array->data.data[i]);
        if(result.as.integer == 1) {
            value_free(&needle);
            value_free(&aval);
            value_int(&array_top(vm->stack), (int64_t)i);
            return;
        }
    }
    value_free(&needle);
    value_free(&aval);
    value_int(&array_top(vm->stack), 0);
}
fn_(insert) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    struct value aval;
    value_copy(&aval, &array_top(vm->stack));
    value_free(&array_top(vm->stack));
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t from_pos = val.as.integer;

    // new val
    val = array_top(vm->stack);

    if(from_pos < 0) {
        value_free(&aval);
        array_push(vm->stack, {0});
        return;
    } else if(from_pos >= (int64_t)aval.as.array->data.length) {
        for(int64_t i = aval.as.array->data.length; i < from_pos+1; i++)
            array_push(aval.as.array->data, {0});
    } else {
        array_push(aval.as.array->data, {0}); // reserve space
    }
    size_t n = (aval.as.array->data.length-from_pos)*sizeof(struct value);
    memmove(&aval.as.array->data.data[from_pos+1],
            &aval.as.array->data.data[from_pos],
            n
    );
    value_copy(&aval.as.array->data.data[from_pos], &val);
    value_free(&aval);

}
fn_(push) {
    assert(nargs == 2);

    struct value aval;
    value_copy(&aval, &array_top(vm->stack));
    value_free(&array_top(vm->stack));
    array_pop(vm->stack);

    array_push(aval.as.array->data, array_top(vm->stack));
    value_free(&array_top(vm->stack));
    value_copy(&array_top(vm->stack), &aval);
    value_free(&aval);
}
fn_(pop) {
    assert(nargs == 1);
    struct value *aval = &array_top(vm->stack);
    if(aval->as.array->data.length == 0) {
        value_free(&array_top(vm->stack));
        return;
    }
    value_free(&aval->as.array->data.data[aval->as.array->data.length-1]);
    aval->as.array->data.length--;
    value_free(&array_top(vm->stack));
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
    //array_pop(vm->stack);
    struct value *aval = &array_top(vm->stack);
    struct value val;
    value_array_n(&val, aval->as.array->data.length);
    for(size_t i = 0; i < aval->as.array->data.length; i++)
        value_copy(&val.as.array->data.data[i], &aval->as.array->data.data[i]);
    val.as.array->data.length = aval->as.array->data.length;
    SORT_ARRAY(val.as.array)
    value_free(aval);
    array_pop(vm->stack);
    array_push(vm->stack, val);
}
fn_(sort_) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    SORT_ARRAY(val->as.array)
}

fn_(map) {
    assert(nargs == 2);

    // string
    struct value aval = array_top(vm->stack);
    array_pop(vm->stack);

    // cb (should be function)
    struct value fn = array_top(vm->stack);
    assert(fn.type == value::TYPE_FN || fn.type == value::TYPE_DICT || fn.type == value::TYPE_NATIVE_FN);
    array_pop(vm->stack);

    if(aval.as.array->data.length == 0) {
        array_push(vm->stack, aval);
        return;
    }

    // init array with same length
    struct value new_val;
    value_array_n(&new_val, aval.as.array->data.length);

    if(fn.type == value::TYPE_FN || fn.type == value::TYPE_DICT) {
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            // setup arguments
            a_arguments args = {
                .data = (struct value*)calloc(1, sizeof(struct value)),
                .length = 1,
                .capacity = 1
            };
            value_copy(&args.data[0], &aval.as.array->data.data[i]);
            // return
            struct value *ret = vm_call(vm, &fn, args);
            value_copy(&new_val.as.array->data.data[i], ret);
            // cleanup
            value_free(&args.data[0]);
            array_free(args);
            value_free(ret);
            array_pop(vm->stack);
        }
    } else {
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            // arguments
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &aval.as.array->data.data[i]);
            // call function
            fn.as.fn(vm, 1);
            new_val.as.array->data.data[i] = array_top(vm->stack);
            // cleanup
            value_free(&array_top(vm->stack));
            array_pop(vm->stack);
        }
    }

    value_free(&aval);
    value_free(&fn);
    array_push(vm->stack, new_val);

}
fn_(filter) {
    assert(nargs == 2);

    // string
    struct value aval = array_top(vm->stack);
    array_pop(vm->stack);

    // cb (should be function)
    struct value fn = array_top(vm->stack);
    assert(fn.type == value::TYPE_FN || fn.type == value::TYPE_NATIVE_FN);
    array_pop(vm->stack);

    if(aval.as.array->data.length == 0) {
        array_push(vm->stack, aval);
        return;
    }

    // init array with same length
    struct value new_val;
    value_array(&new_val);

    if(fn.type == value::TYPE_FN) {
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            // setup arguments
            a_arguments args = {
                .data = (struct value*)calloc(1, sizeof(struct value)),
                .length = 1,
                .capacity = 1
            };
            value_copy(&args.data[0], &aval.as.array->data.data[i]);
            // return
            struct value *ret = vm_call(vm, &fn, args);
            if(value_is_true(ret)) {
                struct value val;
                value_copy(&val, &aval.as.array->data.data[i]);
                array_push(new_val.as.array->data, val);
            }
            // cleanup
            value_free(&args.data[0]);
            array_free(args);
            value_free(ret);
            array_pop(vm->stack);
        }
    } else {
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            // arguments
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &aval.as.array->data.data[i]);
            // call function
            fn.as.fn(vm, 1);
            if(value_is_true(&array_top(vm->stack))) {
                struct value val;
                value_copy(&val, &aval.as.array->data.data[i]);
                array_push(new_val.as.array->data, val);
            }
            // cleanup
            value_free(&array_top(vm->stack));
            array_pop(vm->stack);
        }
    }

    value_free(&aval);
    array_push(vm->stack, new_val);
}
fn_(reduce) {
    assert(nargs == 3);

    // string
    struct value aval = array_top(vm->stack);
    array_pop(vm->stack);

    // cb (should be function)
    struct value fn = array_top(vm->stack);
    assert(fn.type == value::TYPE_FN || fn.type == value::TYPE_NATIVE_FN);
    array_pop(vm->stack);

    if(aval.as.array->data.length == 0) {
        array_push(vm->stack, (struct value){0});
        return;
    }

    // accumulator
    struct value acc;
    value_copy(&acc,&array_top(vm->stack));
    value_free(&array_top(vm->stack));
    array_pop(vm->stack);

    if(fn.type == value::TYPE_FN) {
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            // setup arguments
            a_arguments args = {
                .data = (struct value*)calloc(2, sizeof(struct value)),
                .length = 2,
                .capacity = 2
            };
            value_copy(&args.data[0], &acc);
            value_copy(&args.data[1], &aval.as.array->data.data[i]);
            // return
            struct value *ret = vm_call(vm, &fn, args);
            value_free(&acc);
            value_copy(&acc, ret);
            // cleanup
            value_free(&args.data[0]);
            value_free(&args.data[1]);
            array_free(args);
            value_free(ret);
            array_pop(vm->stack);
        }
    } else {
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            // arguments
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &aval.as.array->data.data[i]);
            array_push(vm->stack, (struct value){0});
            value_copy(&array_top(vm->stack), &acc);
            // call function
            fn.as.fn(vm, 2);
            value_free(&acc);
            value_copy(&acc, &array_top(vm->stack));
            // cleanup
            value_free(&array_top(vm->stack));
            array_pop(vm->stack);
        }
    }

    value_free(&aval);
    array_push(vm->stack, acc);
}

fn_(join) {
    assert(nargs == 2);

    // array
    struct value aval = array_top(vm->stack);
    array_pop(vm->stack);

    // joiner
    struct value sval = array_top(vm->stack);
    assert(sval.type == value::TYPE_STR);
    array_pop(vm->stack);
    const std::string joiner(string_data(sval.as.str));
    value_free(&sval);

    std::string s;
    if(aval.as.array->data.length) {
        assert(aval.as.array->data.data[0].type == value::TYPE_STR);
        s += string_data(aval.as.array->data.data[0].as.str);
    }
    for(size_t i = 1; i < aval.as.array->data.length; i++) {
        assert(aval.as.array->data.data[i].type == value::TYPE_STR);
        s += joiner;
        s += string_data(aval.as.array->data.data[i].as.str);
    }

    value_free(&aval);
    value_str(&sval, s.data());
    array_push(vm->stack, sval);
}
