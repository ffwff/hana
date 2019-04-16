#include <string>
#include "hanayo.h"
#include "vm/src/dict.h"
#include "vm/src/array_obj.h"

std::string hanayo::_to_string(struct value &val) {
    if(val.type == value::TYPE_STR)
        return val.as.str;
    else if(val.type == value::TYPE_INT)
        return std::to_string(val.as.integer);
    else if(val.type == value::TYPE_FLOAT)
        return std::to_string(val.as.floatp);
    else if(val.type == value::TYPE_NATIVE_FN || val.type == value::TYPE_FN)
        return "(function)";
    else if(val.type == value::TYPE_DICT)
        return "(record)";
    else if(val.type == value::TYPE_ARRAY) {
        std::string s = "[";
        if(val.as.array->data.length)
            s += _to_string(val.as.array->data.data[0]);
        for(size_t i = 1; i < val.as.array->data.length; i++)
            s += ", " + _to_string(val.as.array->data.data[i]);
        s += "]";
        return s;
    }
    return "(nil)";
}

void hanayo::_init(struct vm *m) {
    // variables:
    struct value val;
    // # constants
    val = {0}; env_set(m->env, "nil", &val);
    value_int(&val, 0); env_set(m->env, "false", &val);
    value_int(&val, 1); env_set(m->env, "true", &val);
    // # functions
    #define native_function(name) \
    value_native(&val, hanayo::name);  env_set(m->env, #name, &val);
    #define native_function_key(name, key) \
    value_native(&val, hanayo::name);  env_set(m->env, key, &val);
    native_function(print)
    native_function(input)
    // # objects
    #define native_obj_function(key, name) \
    do{ struct value v; value_native(&v, hanayo::name); hmap_set(&val.as.dict->data, key, &v); } while(0)

    value_dict(&val);
    native_obj_function("constructor", string::constructor);
    native_obj_function("bytesize",    string::bytesize);
    native_obj_function("length",      string::length);
    native_obj_function("delete",      string::delete_);
    native_obj_function("copy",        string::copy);
    native_obj_function("at",          string::at);
    native_obj_function("index",       string::index);
    native_obj_function("insert",      string::insert);
    env_set(m->env, "string", &val);
    value_free(&val);
    m->dstr = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", integer::constructor);
    env_set(m->env, "integer", &val);
    value_free(&val);
    m->dint = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", float_::constructor);
    native_obj_function("round",       float_::round);
    env_set(m->env, "float", &val);
    value_free(&val);
    m->dfloat = val.as.dict;

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
    env_set(m->env, "array", &val);
    value_free(&val);
    m->darray = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", record::constructor);
    env_set(m->env, "record", &val);
    value_free(&val);
}
