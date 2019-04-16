#include <cassert>
#include "hanayo.h"
#include "vm/src/array_obj.h"

#define fn(name) void hanayo::array::name(struct vm *vm, int nargs)

fn(constructor) {
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

fn(length) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    int64_t length = val->as.array->data.length;
    value_free(val);
    value_int(val, length);
}
fn(delete_) {
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
fn(copy) {
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
fn(at) {
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
fn(index) {
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
fn(insert) {
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
fn(push) {
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
fn(pop) {
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
fn(sort) {
    assert(nargs == 1);
    //array_pop(vm->stack);
    struct value *aval = &array_top(vm->stack);
    struct value val;
    value_array_n(&val, aval->as.array->data.length);
    for(size_t i = 0; i < aval->as.array->data.length; i++)
        value_copy(&val.as.array->data.data[i], &aval->as.array->data.data[i]);
    val.as.array->data.length = aval->as.array->data.length;
    qsort(val.as.array->data.data, val.as.array->data.length, sizeof(struct value), [](const void *a, const void *b) -> int {
        const struct value *va = (const struct value *)a,
          *vb = (const struct value *)b;
          struct value ret = {0};
          value_lt(&ret, va, vb);
          if(ret.as.integer == 1) return -1;
          value_eq(&ret, va, vb);
          if(ret.as.integer == 1) return 0;
          return 1;
    });
    value_free(aval);
    array_pop(vm->stack);
    array_push(vm->stack, val);
}
fn(sort_) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    qsort(val->as.array->data.data, val->as.array->data.length, sizeof(struct value), [](const void *a, const void *b) -> int {
        const struct value *va = (const struct value *)a,
          *vb = (const struct value *)b;
          struct value ret = {0};
          value_lt(&ret, va, vb);
          if(ret.as.integer == 1) return -1;
          value_eq(&ret, va, vb);
          if(ret.as.integer == 1) return 0;
          return 1;
    });
}
