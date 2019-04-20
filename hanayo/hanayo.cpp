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
    if(val.type == value::TYPE_STR)
        return strdup(string_data(val.as.str));
    else if(val.type == value::TYPE_INT) {
        char dummy[1];
        const size_t siz = snprintf(dummy, 1, "%ld", val.as.integer);
        char *s = (char*)malloc(siz+1);
        sprintf(s, "%ld", val.as.integer);
        return s;
    } else if(val.type == value::TYPE_FLOAT) {
        char dummy[1];
        const size_t siz = snprintf(dummy, 1, "%f", val.as.floatp);
        char *s = (char*)malloc(siz+1);
        sprintf(s, "%f", val.as.floatp);
        return s;
    }
    else if(val.type == value::TYPE_NATIVE_FN || val.type == value::TYPE_FN)
        return strdup("(function)");
    else if(val.type == value::TYPE_DICT)
        return strdup("(record)");
    else if(val.type == value::TYPE_ARRAY) {
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
    auto sval = _arg(vm, value::TYPE_STR);
    auto script = sval.as.str;

    // generate ast & emit
    auto ast = hanayo_parse(string_data(script));
    auto target_ip = vm->code.length;
    hanayo_ast_emit(ast, vm);
    hanayo_free_ast(ast);

    // save state, execute then return
    auto ip = vm->ip;
    vm->ip = target_ip;
    vm_execute(vm);
    vm->ip = ip;

    // cleanup
    value_free(&sval);
    array_push(vm->stack, {0});
}

void hanayo::_init(struct vm *m) {
    // variables:
    struct value val;
    // # constants
    val = {0}; hmap_set(&m->globalenv, "nil", &val);
    // # functions
#define native_function(name) \
    value_native(&val, hanayo::name);  hmap_set(&m->globalenv, #name, &val);
#define native_function_key(name, key) \
    value_native(&val, hanayo::name);  hmap_set(&m->globalenv, key, &val);

    // ## io
    native_function(print)
    native_function(input)
    native_function(fopen)
    native_function(fread)
    native_function(fwrite)
    native_function(eval)

    // # objects
#define native_obj_function(key, name) \
    do{ struct value v; value_native(&v, hanayo::name); hmap_set(&val.as.dict->data, key, &v); } while(0)

    // ## strings
    value_dict(&val);
    native_obj_function("constructor", string::constructor);
    native_obj_function("bytesize",    string::bytesize);
    native_obj_function("length",      string::length);
    native_obj_function("delete",      string::delete_);
    native_obj_function("copy",        string::copy);
    native_obj_function("at",          string::at);
    native_obj_function("index",       string::index);
    native_obj_function("insert",      string::insert);
    native_obj_function("split",       string::split);
    hmap_set(&m->globalenv, "string", &val);
    value_free(&val);
    m->dstr = val.as.dict;

    // ## integers
    value_dict(&val);
    native_obj_function("constructor", integer::constructor);
    hmap_set(&m->globalenv, "integer", &val);
    value_free(&val);
    m->dint = val.as.dict;

    // ## floats
    value_dict(&val);
    native_obj_function("constructor", float_::constructor);
    native_obj_function("round",       float_::round);
    hmap_set(&m->globalenv, "float", &val);
    value_free(&val);
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
    hmap_set(&m->globalenv, "array", &val);
    value_free(&val);
    m->darray = val.as.dict;
}

struct value hanayo::_arg(struct vm *vm, value::value_type type) {
    struct value val = array_top(vm->stack);
    assert(val.type == type);
    array_pop(vm->stack);
    return val;
}
