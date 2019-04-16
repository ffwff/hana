#include <string>
#include <cassert>
#include "hanayo.h"

#define fn(name) void hanayo::string::name(struct vm *vm, int nargs)

fn(constructor) {
    assert(nargs == 1);
    struct value val = array_top(vm->stack);
    auto s = hanayo::_to_string(val);
    value_free(&val);
    array_pop(vm->stack);
    value_str(&val, s.data());
    array_push(vm->stack, val);
    vm_print_stack(vm);
}

fn(bytesize) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    int64_t length = strlen(val->as.str);
    value_free(val);
    array_pop(vm->stack);
    struct value intv;
    value_int(&intv, length);
    array_push(vm->stack, intv);
}
fn(length) { // TODO unicode char length
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    int64_t length = strlen(val->as.str);
    value_free(val);
    array_pop(vm->stack);
    struct value intv;
    value_int(&intv, length);
    array_push(vm->stack, intv);
}
fn(delete_) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    val = array_top(vm->stack);
    std::string s(val.as.str);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    array_pop(vm->stack);
    int64_t from_pos = val.as.integer;

    // n chars
    val = array_top(vm->stack);
    array_pop(vm->stack);
    int64_t nchars = val.as.integer;

    s.erase(from_pos, nchars);
    value_str(&val, s.data());
    array_push(vm->stack, val);
}
fn(copy) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    val = array_top(vm->stack);
    std::string s(val.as.str);
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

    char str[nchars+1];
    s.copy(str, nchars, from_pos);
    str[nchars] = 0;
    value_str(&val, str);
    array_push(vm->stack, val);
}
fn(at) {
    assert(nargs == 2);
    struct value val = {0};

    // string
    val = array_top(vm->stack);
    std::string s(val.as.str);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t index = val.as.integer;

    if(index < (int64_t)(s.size()) && index >= 0) {
        char str[2] = { s[index], 0 };
        value_str(&val, str);
        array_push(vm->stack, val);
    } else {
        array_push(vm->stack, {0});
    }
}
fn(index) {
    assert(nargs == 2);
    struct value val = {0};

    // string
    val = array_top(vm->stack);
    std::string src(val.as.str);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_STR);
    std::string needle(val.as.str);
    value_free(&val);
    array_pop(vm->stack);

    size_t index = src.find(needle);
    if(index == std::string::npos) {
        value_int(&val, -1);
    } else {
        value_int(&val, (int64_t)index);
    }
    array_push(vm->stack, val);
}
fn(insert) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    val = array_top(vm->stack);
    std::string s(val.as.str);
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t from_pos = val.as.integer;

    val = array_top(vm->stack);
    assert(val.type == value::TYPE_STR);
    s.insert(from_pos, val.as.str);
    value_free(&val);

    value_str(&array_top(vm->stack), s.data());
}
