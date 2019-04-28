#include <assert.h>
#include <string.h>
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"
#include "hanayo.h"

struct string_header *string_alloc(size_t n) {
    struct string_header *s = (struct string_header *)malloc(sizeof(struct string_header)+n+1);
    s->length = n;
    s->refs = 1;
    return s;
}

#define fn(name) void hanayo::string::name(struct vm *vm, int nargs)

// static
fn(constructor) {
    assert(nargs == 1);
    struct value val = array_top(vm->stack);
    auto s = hanayo::_to_string(val);
    value_free(&val);
    array_pop(vm->stack);
    value_str(&val, s); free(s);
    array_push(vm->stack, val);
}
fn(reserve) {
    assert(nargs == 1);
    auto len = _arg(vm, value::TYPE_INT).as.integer;
    struct value val;
    value_str_reserve(&val, len);
    array_push(vm->stack, val);
}

// instance
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

    auto length = string_len(val.as.str);
    struct string_header *s = string_alloc(length);
    char *ss = string_data(s);
    strcpy(ss, string_data(val.as.str));

    value_free(&val);
    array_pop(vm->stack);

    // from pos
    val = _arg(vm, value::TYPE_INT);
    int64_t from_pos = val.as.integer;

    // n chars
    val = _arg(vm, value::TYPE_INT);
    int64_t nchars = val.as.integer;

    // checks
    assert(from_pos >= 0 && from_pos < (int64_t)length);
    assert((from_pos+nchars) >= 0 && (from_pos+nchars) < (int64_t)length);

    // do it
    memmove(ss+from_pos, ss+from_pos+nchars, length-from_pos-nchars+1);
    value_strmov(&val, s);
    array_push(vm->stack, val);
}
fn(copy) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    auto sval = array_top(vm->stack);
    auto length = string_len(sval.as.str);
    array_pop(vm->stack);

    // from pos
    val = _arg(vm, value::TYPE_INT);
    int64_t from_pos = val.as.integer;

    // n chars
    val = _arg(vm, value::TYPE_INT);
    int64_t nchars = val.as.integer;

    // checks
    assert(from_pos >= 0 && from_pos < (int64_t)length);
    assert((from_pos+nchars) >= 0 && (from_pos+nchars) < (int64_t)length);

    // do it
    struct string_header *s = string_alloc(nchars);
    char *ss = string_data(s);
    strncpy(ss, string_data(sval.as.str)+from_pos, nchars);
    ss[nchars] = 0;

    value_free(&sval);
    value_strmov(&val, s);
    array_push(vm->stack, val);
}
fn(at) {
    assert(nargs == 2);
    struct value val = {0};

    // string
    auto sval = array_top(vm->stack);
    array_pop(vm->stack);

    // from pos
    val = _arg(vm, value::TYPE_INT);
    int64_t index = val.as.integer;

    if(index < (int64_t)string_len(sval.as.str) && index >= 0) {
        char str[2] = { string_at(sval.as.str, index), 0 };
        value_str(&val, str);
        array_push(vm->stack, val);
    } else {
        array_push(vm->stack, {0});
    }
    value_free(&sval);
}
fn(index) {
    assert(nargs == 2);

    // string
    auto hval = array_top(vm->stack);
    auto haystack = string_data(hval.as.str);
    array_pop(vm->stack);

    // from pos
    auto nval = _arg(vm, value::TYPE_STR);
    auto needle = string_data(nval.as.str);

    auto occ = strstr(haystack, needle);
    struct value val = {0};
    if(occ == nullptr) {
        value_int(&val, -1);
    } else {
        size_t index = (size_t)occ-(size_t)haystack;
        value_int(&val, (int64_t)index);
    }
    value_free(&hval);
    value_free(&nval);
    array_push(vm->stack, val);
}
fn(insert) {
    assert(nargs == 3);

    struct value val = {0};

    // string
    auto dval = array_top(vm->stack);
    auto dst = string_data(dval.as.str);
    array_pop(vm->stack);

    // from pos
    val = _arg(vm, value::TYPE_INT);
    int64_t from_pos = val.as.integer;

    // string
    auto sval = _arg(vm, value::TYPE_STR);
    auto src = string_data(sval.as.str);

    // checks
    assert(from_pos >= 0 && from_pos < (int64_t)string_len(dval.as.str));

    // doit
    struct string_header *s = string_alloc(string_len(dval.as.str)+string_len(sval.as.str));
    char *ss = string_data(s);
    strncpy(ss, dst, from_pos); // dst lower half
    memcpy(ss+from_pos, src, strlen(src)); // src
    strncpy(ss+from_pos+string_len(sval.as.str),
            dst+from_pos, string_len(dval.as.str)-from_pos); // dst upper half
    ss[string_len(dval.as.str)+string_len(sval.as.str)] = 0;

    value_free(&dval);
    value_free(&sval);
    value_strmov(&val, s);
    array_push(vm->stack, val);
}

fn(split) {
    assert(nargs == 2);

    // string
    auto sval = array_top(vm->stack);
    auto str = strdup(string_data(sval.as.str));
    array_pop(vm->stack);
    value_free(&sval);

    // delim
    auto dval = _arg(vm, value::TYPE_STR);
    auto delim = string_data(dval.as.str);

    struct value val = {0};
    value_array(&val);
    if(string_len(dval.as.str) == 0) { // turns string to array of chars if called with ""
        for(size_t i = 0; i < string_len(sval.as.str); i++) {
            char ch[2] = { string_at(sval.as.str, i), 0 };
            struct value vtoken;
            value_str(&vtoken, ch);
            array_push(val.as.array->data, vtoken);
        }
    } else {
        char *saveptr = nullptr;
        char *tok = nullptr;
        for(tok = strtok_r(str, delim, &saveptr);
            tok != nullptr; tok = strtok_r(nullptr, delim, &saveptr)) {
            struct value vtoken;
            value_str(&vtoken, tok);
            array_push(val.as.array->data, vtoken);
        }
    }

    free(str);
    value_free(&dval);
    array_push(vm->stack, val);
}

fn(startswith) {
    auto sval = array_top(vm->stack);
    auto ls = string_len(sval.as.str);
    array_pop(vm->stack);

    auto dval = _arg(vm, value::TYPE_STR);
    auto lt = string_len(dval.as.str);
    bool sw = ls >= lt && strncmp(string_data(sval.as.str), string_data(dval.as.str), string_len(dval.as.str)) == 0;

    value_free(&sval);
    value_free(&dval);

    struct value val = {0};
    value_int(&val, sw);
    array_push(vm->stack, val);
}

fn(endswith) {
    auto sval = array_top(vm->stack);
    auto ls = string_len(sval.as.str);
    array_pop(vm->stack);

    auto dval = _arg(vm, value::TYPE_STR);
    auto lt = string_len(dval.as.str);

    bool sw = ls >= lt && memcmp(string_data(dval.as.str), string_data(sval.as.str)+(ls-lt), lt) == 0;

    value_free(&sval);
    value_free(&dval);

    struct value val = {0};
    value_int(&val, sw);
    array_push(vm->stack, val);
}

fn(shrink_) {
    auto *sval = &array_top(vm->stack);
    auto len = _arg(vm, value::TYPE_INT).as.integer;
    if(sval->as.str->length < len) return;
    sval->as.str->length = len;
    char *s = string_data(sval->as.str);
    s[len] = 0;
}
