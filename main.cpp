#include <iostream>
#include <memory>
#include "src/scriptparser.h"
#include "vm/src/vm.h"
#include "vm/src/dict.h"

namespace hanayo {

static std::string to_string(struct value &val) {
    if(val.type == value::TYPE_STR)
        return val.as.str;
    else if(val.type == value::TYPE_INT)
        return std::to_string(val.as.integer);
    else if(val.type == value::TYPE_FLOAT)
        return std::to_string(val.as.floatp);
    else if(val.type == value::TYPE_NATIVE_FN || val.type == value::TYPE_FN)
        return "[function]";
    else if(val.type == value::TYPE_DICT)
        return "[dictionary]";
    return "[nil]";
}

#define fn(name) void name(struct vm *vm, int nargs)

fn(print) {
    int written = 0;
    while(nargs--) {
        struct value val = array_top(vm->stack);
        auto s = to_string(val);
        written += fputs(s.data(), stdout);
        value_free(&val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
    array_push(vm->stack, val);
}

// data types
namespace string {

    fn(constructor) {
        assert(nargs == 1);
        struct value val = array_top(vm->stack);
        auto s = to_string(val);
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
}

namespace integer {
    fn(constructor) {
    }
}

namespace float_ {
    fn(constructor) {
    }
}

namespace record {
    fn(constructor) {
        struct value val; value_dict(&val);
        array_push(vm->stack, val);
    }
}

#undef fn

}

int main(int argc, char **argv) {
    if(argc != 2) {
        std::cout << "expects at least 2!";
        return 1;
    }
    // parser
    Hana::ScriptParser p;
    p.loadf(argv[1]);
    auto ast = std::unique_ptr<Hana::AST::AST>(p.parse());
#if !defined(NOLOG) && defined(DEBUG)
    ast->print();
#endif

    // virtual machine
    struct vm m; vm_init(&m);
    // variables:
    struct value val;
    // # constants
    val = {0}; env_set(m.env, "nil", &val);
    value_int(&val, 0); env_set(m.env, "false", &val);
    value_int(&val, 1); env_set(m.env, "true", &val);
    // # functions
#define native_function(name) \
    value_native(&val, hanayo::name);  env_set(m.env, #name, &val);
#define native_function_key(name, key) \
    value_native(&val, hanayo::name);  env_set(m.env, key, &val);
    native_function(print)
    // # objects
#define native_obj_function(key, name) \
    do{ struct value v; value_native(&v, hanayo::name); map_set(&val.as.dict->data, key, &v); } while(0)

    value_dict(&val);
    native_obj_function("constructor", string::constructor);
    native_obj_function("bytesize",    string::bytesize);
    native_obj_function("length",      string::length);
    native_obj_function("delete",      string::delete_);
    native_obj_function("copy",        string::copy);
    native_obj_function("at",          string::at);
    native_obj_function("index",       string::index);
    env_set(m.env, "string", &val);
    value_free(&val);
    m.dstr = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", integer::constructor);
    env_set(m.env, "integer", &val);
    value_free(&val);
    m.dint = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", float_::constructor);
    env_set(m.env, "float", &val);
    value_free(&val);
    m.dfloat = val.as.dict;

    // emit bytecode
    ast->emit(&m);
    array_push(m.code, OP_HALT);

    // execute!
    //while(vm_step(&m)) std::cin.get();
    vm_execute(&m);

    // cleanup
    vm_free(&m);
    return 0;
}
