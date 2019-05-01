#include <string.h>
#include <stdio.h>
#include "hanayo.h"
#include "vm/src/string_.h"
#include "vm/src/dict.h"
#include "vm/src/array_obj.h"
#include "src/binding.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

// helpers
char *hanayo::_to_string(struct value &val) {
    if(val.type == TYPE_STR)
        return strdup(string_data(val.as.str));
    else if(val.type == TYPE_INT) {
        char dummy[1];
        const size_t siz = snprintf(dummy, 1, "%ld", val.as.integer);
        char *s = (char*)malloc(siz+1);
        sprintf(s, "%ld", val.as.integer);
        return s;
    } else if(val.type == TYPE_FLOAT) {
        char dummy[1];
        const size_t siz = snprintf(dummy, 1, "%f", val.as.floatp);
        char *s = (char*)malloc(siz+1);
        sprintf(s, "%f", val.as.floatp);
        return s;
    }
    else if(val.type == TYPE_NATIVE_FN || val.type == TYPE_FN)
        return strdup("(function)");
    else if(val.type == TYPE_DICT)
        return strdup("(record)");
    else if(val.type == TYPE_ARRAY) {
        const auto joiner = ", ";
        const auto joiner_len = strlen(joiner);

        size_t len = 1;
        char *s = (char*)malloc(len+1);
        s[0] = '['; s[1] = 0;
        const auto END = 1;
        if(val.as.array->data.length) {
            auto ss = _to_string(val.as.array->data.data[0]);
            len += strlen(ss); s = (char*)realloc(s, len+END+1);
            strcat(s, ss);
            free(ss);
        }
        for(size_t i = 1; i < val.as.array->data.length; i++) {
            auto ss = _to_string(val.as.array->data.data[i]);
            len += joiner_len + strlen(ss); s = (char*)realloc(s, len+END+1);
            strcat(s, joiner);
            strcat(s, ss);
            free(ss);
        }
        s[len] = ']';
        s[len+1] = 0;
        return s;
    }
    return strdup("(nil)");
}

// fns
fn(eval) {
    auto sval = _arg(vm, TYPE_STR);
    const auto script = string_data(sval.v.as.str);
    Value retval;

    // generate ast & emit
    auto ast = hana_parse(script);
    if(ast == nullptr) {
        value_int(retval, 0);
    } else {
        const auto target_ip = vm->code.length;
        hana_ast_emit(ast, vm);
        hana_free_ast(ast);

        // save state, execute then return
        const auto ip = vm->ip;
        vm->ip = target_ip;
        vm_execute(vm);
        vm->ip = ip;
        value_int(retval, 1);
    }

    _push(vm, retval);
}

fn(exit) {
    const auto code = _arg(vm, TYPE_INT);
    ::exit(code.v.as.integer);
}

void hanayo::_init(struct vm *m) {
    // variables:
    struct value val;
    // # constants
    val = {0}; hmap_set(&m->globalenv, "nil", &val);
    value_float(&val, 0.0/0.0); hmap_set(&m->globalenv, "nan", &val);
    value_float(&val, 1.0/0.0); hmap_set(&m->globalenv, "inf", &val);
    value_int(&val, 1); hmap_set(&m->globalenv, "true", &val);
    value_int(&val, 0); hmap_set(&m->globalenv, "false", &val);

    // # functions
#define native_function(name) \
    value_native(&val, hanayo::name);  hmap_set(&m->globalenv, #name, &val);
#define native_function_key(name, key) \
    value_native(&val, hanayo::name);  hmap_set(&m->globalenv, key, &val);
    // # objects
#define native_obj_function(key, name) \
    do{ struct value v; value_native(&v, hanayo::name); hmap_set(&val.as.dict->data, key, &v); } while(0)

    // ## io
    native_function(print)
    native_function(println)
    native_function(input)
    native_function(eval)
    native_function(exit)

    // ffi
    value_dict(&val);
    native_obj_function("Function"   , ffi::function);
    {
        struct value rc; value_dict(&rc);
        hanayo::ffi::rcpointer::prototype = rc;

        struct value v; value_native(&v, hanayo::ffi::rcpointer::constructor);
        dict_set(rc.as.dict, "constructor", &v);

        dict_set(val.as.dict, "RcPointer", &rc);
    }
    native_obj_function("call",        ffi::call);
#define X(x) {struct value v; value_int(&v, hanayo::ffi::type::x); dict_set(val.as.dict, # x, &v);}
    X(UInt8); X(Int8); X(UInt16); X(Int16); X(UInt32); X(Int32); X(UInt64); X(Int64);
    X(Float32); X(Float64); X(UChar); X(Char); X(UShort); X(Short); X(ULong); X(Long);
    X(Pointer); X(String); X(Void);
    hmap_set(&m->globalenv, "Cffi", &val);

    // ## strings
    value_dict(&val);
    native_obj_function("constructor", string::constructor);
    native_obj_function("reserve",     string::reserve);
    native_obj_function("bytesize",    string::bytesize);
    native_obj_function("length",      string::length);
    native_obj_function("delete",      string::delete_);
    native_obj_function("copy",        string::copy);
    native_obj_function("at",          string::at);
    native_obj_function("index",       string::index);
    native_obj_function("insert",      string::insert);
    native_obj_function("split",       string::split);
    native_obj_function("startswith?", string::startswith);
    native_obj_function("endswith?",   string::endswith);
    native_obj_function("shrink!",     string::shrink_);
    hmap_set(&m->globalenv, "String", &val);
    m->dstr = val.as.dict;

    // ## integers
    value_dict(&val);
    native_obj_function("constructor", integer::constructor);
    hmap_set(&m->globalenv, "Int", &val);
    m->dint = val.as.dict;

    // ## floats
    value_dict(&val);
    native_obj_function("constructor", float_::constructor);
    native_obj_function("round",       float_::round);
    hmap_set(&m->globalenv, "Float", &val);
    m->dfloat = val.as.dict;

    // ## arrays
    value_dict(&val);
    native_obj_function("constructor", array::constructor);
    native_obj_function("length",      array::length);
    native_obj_function("delete",      array::delete_);
    native_obj_function("copy",        array::copy);
    native_obj_function("at",          array::at);
    native_obj_function("index",       array::index);
    native_obj_function("insert",      array::insert);
    native_obj_function("push",        array::push);
    native_obj_function("pop",         array::pop);
    native_obj_function("sort",        array::sort);
    native_obj_function("sort!",       array::sort_);
    native_obj_function("map",         array::map);
    native_obj_function("filter",      array::filter);
    native_obj_function("reduce",      array::reduce);
    native_obj_function("join",        array::join);
    hmap_set(&m->globalenv, "Array", &val);
    m->darray = val.as.dict;

    // ## records
    value_dict(&val);
    native_obj_function("constructor", record::constructor);
    native_obj_function("is_record?",  record::is_record);
    native_obj_function("keys",        record::keys);
    native_obj_function("copy",        record::copy);
    hmap_set(&m->globalenv, "Record", &val);
}

// helpers
hanayo::Value hanayo::_top(struct vm *vm) { return array_top(vm->stack); }
hanayo::Value hanayo::_pop(struct vm *vm) {
    auto v = array_top(vm->stack);
    array_pop(vm->stack);
    Value vv; vv.v = v;
    return vv;
}
void hanayo::_push(struct vm *vm, Value &val) {
    array_push(vm->stack, val);
    val.v.type = TYPE_NIL;
}

hanayo::Value hanayo::_arg(struct vm *vm, uint8_t type) {
    struct value v = array_top(vm->stack);
    assert(v.type == type);
    Value val; val.v = v;
    array_pop(vm->stack);
    return val;
}
