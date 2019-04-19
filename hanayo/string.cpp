#include <string>
#include <cassert>
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"
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
}

fn(bytesize) {
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    int64_t length = string_len(val->as.str);
    value_free(val);
    array_pop(vm->stack);
    struct value intv;
    value_int(&intv, length);
    array_push(vm->stack, intv);
}
fn(length) { // TODO unicode char length
    assert(nargs == 1);
    struct value *val = &array_top(vm->stack);
    int64_t length = string_len(val->as.str);
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
    std::string s(string_data(val.as.str));
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
    std::string s(string_data(val.as.str));
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
    std::string s(string_data(val.as.str));
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
    std::string s(string_data(val.as.str));
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_STR);
    std::string needle(string_data(val.as.str));
    value_free(&val);
    array_pop(vm->stack);

    size_t index = s.find(needle);
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
    std::string s(string_data(val.as.str));
    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = array_top(vm->stack);
    assert(val.type == value::TYPE_INT);
    array_pop(vm->stack);
    int64_t from_pos = val.as.integer;

    val = array_top(vm->stack);
    assert(val.type == value::TYPE_STR);
    s.insert(from_pos, string_data(val.as.str));
    value_free(&val);

    value_str(&array_top(vm->stack), s.data());
}

fn(split) {
    assert(nargs == 2);

    struct value val = {0};

    // string
    val = array_top(vm->stack);
    std::string s(string_data(val.as.str));
    value_free(&val);
    array_pop(vm->stack);

    // delim
    val = array_top(vm->stack);
    const std::string delim(string_data(val.as.str));
    value_free(&val);
    array_pop(vm->stack);

    value_array(&val);
    if(delim.empty()) { // turns string to array if called with ""
        for(size_t i = 0; i < s.size(); i++) {
            char ch[2] = { s[i], 0 };
            struct value vtoken;
            value_str(&vtoken, ch);
            array_push(val.as.array->data, vtoken);
        }
    } else {
        size_t pos = 0;
        while ((pos = s.find(delim)) != std::string::npos) {
            const std::string token = s.substr(0, pos);
            struct value vtoken;
            value_str(&vtoken, token.data());
            array_push(val.as.array->data, vtoken);
            s.erase(0, pos + delim.length());
        }
        if(!s.empty()) {
            struct value vtoken;
            value_str(&vtoken, s.data());
            array_push(val.as.array->data, vtoken);
        }
    }

    array_push(vm->stack, val);
}
