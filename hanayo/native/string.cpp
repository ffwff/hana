#include <assert.h>
#include <string.h>
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"
#include "vm/src/value.h"
#include "hanayo.h"

struct string_header *string_alloc(size_t n) {
    struct string_header *s = (struct string_header *)malloc(sizeof(struct string_header)+n+1);
    s->length = n;
    return s;
}

#define fn(name) void hanayo::string::name(struct vm *vm, int nargs)

// static
fn(constructor) {
    assert(nargs == 1);
    auto s = hanayo::_to_string(_pop(vm));
    Value val;
    value_str(val, s); free(s);
    _push(vm, val);
}
fn(reserve) {
    assert(nargs == 1);
    const auto len = _arg(vm, TYPE_INT).v.as.integer;
    Value val; value_str_reserve(val, len);
    _push(vm, val);
}

// instance
fn(bytesize) {
    assert(nargs == 1);
    Value val = _arg(vm, TYPE_STR);
    Value retval; value_int(retval, string_len(val.v.as.str));
    _push(vm, retval);
}
fn(length) { // TODO unicode char length
    assert(nargs == 1);
    Value val = _arg(vm, TYPE_STR);
    Value retval; value_int(retval, string_len(val.v.as.str));
    _push(vm, retval);
}
fn(delete_) {
    assert(nargs == 3);

    // string
    struct string_header *s = nullptr;
    char *ss = nullptr;
    int64_t length;
    {
        Value val = _arg(vm, TYPE_STR);

        length = string_len(val.v.as.str);
        s = string_alloc(length);
        ss = string_data(s);
        strcpy(ss, string_data(val.v.as.str));
    }

    // arguments
    const int64_t from_pos = _arg(vm, TYPE_INT).v.as.integer;
    const int64_t nchars = _arg(vm, TYPE_INT).v.as.integer;

    // checks
    assert(from_pos >= 0 && from_pos < (int64_t)length);
    assert((from_pos+nchars) >= 0 && (from_pos+nchars) < (int64_t)length);

    // do it
    memmove(ss+from_pos, ss+from_pos+nchars, length-from_pos-nchars+1);
    {
        Value val; value_strmov(val, s);
        _push(vm, val); // TODO
    }
}
fn(copy) {
    assert(nargs == 3);

    // string
    const auto sval = _arg(vm, TYPE_STR);
    const int64_t length = string_len(sval.v.as.str);

    const int64_t from_pos = _arg(vm, TYPE_INT).v.as.integer;
    const int64_t nchars = _arg(vm, TYPE_INT).v.as.integer;

    // checks
    assert(from_pos >= 0 && from_pos < (int64_t)length);
    assert((from_pos+nchars) >= 0 && (from_pos+nchars) < (int64_t)length);

    // do it
    struct string_header *s = string_alloc(nchars);
    char *ss = string_data(s);
    strncpy(ss, string_data(sval.v.as.str)+from_pos, nchars);
    ss[nchars] = 0;

    {
        Value retval; value_strmov(retval, s);
        _push(vm, retval);
    }
}
fn(at) {
    assert(nargs == 2);

    // args
    const auto sval = _arg(vm, TYPE_STR);
    const int64_t index = _arg(vm, TYPE_INT).v.as.integer;

    if(index < (int64_t)string_len(sval.v.as.str) && index >= 0) {
        char str[2] = { string_at(sval.v.as.str, index), 0 };
        Value retval; value_str(retval, str);
        _push(vm, retval);
    } else {
        Value retval;
        _push(vm, retval);
    }
}
fn(index) {
    assert(nargs == 2);

    // string
    const auto hval = _arg(vm, TYPE_STR);
    const char *haystack = string_data(hval.v.as.str);

    // needle
    const auto nval = _arg(vm, TYPE_STR);
    const char *needle = string_data(nval.v.as.str);

    const auto occ = strstr(haystack, needle);
    if(occ == nullptr) {
        Value retval; value_int(retval, -1);
        _push(vm, retval);
    } else {
        const size_t index = (size_t)occ-(size_t)haystack;
        Value retval; value_int(retval, (int64_t)index);
        _push(vm, retval);
    }
}
fn(insert) {
    assert(nargs == 3);

    // string
    const auto dval = _arg(vm, TYPE_STR);
    const char *dst = string_data(dval.v.as.str);

    // from pos
    const int64_t from_pos = _arg(vm, TYPE_INT).v.as.integer;

    // string
    const auto sval = _arg(vm, TYPE_STR);
    const char *src = string_data(sval.v.as.str);

    // checks
    assert(from_pos >= 0 && from_pos < (int64_t)string_len(dval.v.as.str));

    // doit
    struct string_header *s = string_alloc(string_len(dval.v.as.str)+string_len(sval.v.as.str));
    char *ss = string_data(s);
    strncpy(ss, dst, from_pos); // dst lower half
    memcpy(ss+from_pos, src, strlen(src)); // src
    strncpy(ss+from_pos+string_len(sval.v.as.str),
            dst+from_pos, string_len(dval.v.as.str)-from_pos); // dst upper half
    ss[string_len(dval.v.as.str)+string_len(sval.v.as.str)] = 0;

    {
        Value retval;
        value_strmov(retval, s);
        _push(vm, retval);
    }
}

fn(split) {
    assert(nargs == 2);
    // string
    char *str = nullptr;
    size_t sval_len;
    {
        const auto sval = _arg(vm, TYPE_STR);
        str = strdup(string_data(sval.v.as.str));
        sval_len = string_len(sval.v.as.str);
    }

    // delim
    const auto dval = _arg(vm, TYPE_STR);
    const char *delim = string_data(dval.v.as.str);

    Value retval; value_array(retval);
    if(sval_len == 0) { // turns string to array of chars if called with ""
        for(size_t i = 0; i < sval_len; i++) {
            const char ch[2] = { string_at(str, i), 0 };
            struct value vtoken;
            value_str(&vtoken, ch);
            array_push(retval.v.as.array->data, vtoken);
        }
    } else {
        char *saveptr = nullptr;
        char *tok = nullptr;
        for(tok = strtok_r(str, delim, &saveptr);
            tok != nullptr; tok = strtok_r(nullptr, delim, &saveptr)) {
            struct value vtoken;
            value_str(&vtoken, tok);
            array_push(retval.v.as.array->data, vtoken);
        }
    }

    free(str);
    _push(vm, retval);
}

fn(startswith) {
    const auto sval = _arg(vm, TYPE_STR);
    const size_t ls = string_len(sval.v.as.str);

    const auto dval = _arg(vm, TYPE_STR);
    const size_t lt = string_len(dval.v.as.str);

    const bool sw = ls >= lt && strncmp(string_data(sval.v.as.str), string_data(dval.v.as.str), string_len(dval.v.as.str)) == 0;

    Value retval; value_int(retval, sw);
    _push(vm, retval);
}

fn(endswith) {
    const auto sval = _arg(vm, TYPE_STR);
    const size_t ls = string_len(sval.v.as.str);

    const auto dval = _arg(vm, TYPE_STR);
    const size_t lt = string_len(dval.v.as.str);

    const bool ew = ls >= lt && memcmp(string_data(dval.v.as.str), string_data(sval.v.as.str)+(ls-lt), lt) == 0;

    Value retval; value_int(retval, ew);
    _push(vm, retval);
}

fn(shrink_) {
    if(nargs == 1) {
        auto sval = _arg(vm, TYPE_STR);
        const auto len = strlen(string_data(sval.v.as.str));
        sval.v.as.str->length = len;
        _push(vm, sval);
    } else if(nargs == 2) {
        auto sval = _arg(vm, TYPE_STR);
        const auto len = _arg(vm, TYPE_INT).v.as.integer;
        if(sval.v.as.str->length < len) return;
        sval.v.as.str->length = len;
        char *s = string_data(sval.v.as.str);
        s[len] = 0;
        _push(vm, sval);
    } else assert(0);
}
