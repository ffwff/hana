#include <iostream>
#include <memory>
#include "src/scriptparser.h"
#include "vm/src/vm.h"

namespace hanayo {

void print(struct vm *vm, int nargs) {
    int written = 0;
    while(nargs--) {
        struct value *val = &array_top(vm->stack);
        if(val->type == value::TYPE_STR)
            written += printf("%s", val->as.str);
        else if(val->type == value::TYPE_INT)
            written += printf("%ld", val->as.integer);
        else if(val->type == value::TYPE_FLOAT)
            written += printf("%f", val->as.floatp);
        else if(val->type == value::TYPE_NATIVE_FN || val->type == value::TYPE_FN)
            written += printf("[function]");
        else if(val->type == value::TYPE_DICT)
            written += printf("[dictionary]");
        else
            written += printf("[nil]");
        value_free(val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
    array_push(vm->stack, val);
}

// data types
void string(struct vm *vm, int nargs) {

}
void int_(struct vm *vm, int nargs) {

}
void float_(struct vm *vm, int nargs) {

}

void record(struct vm *vm, int nargs) {
    struct value val; value_dict(&val);
    array_push(vm->stack, val);
}

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
#if defined(NOLOG) && !defined(RELEASE)
    ast->print();
#endif

    // virtual machine
    struct vm m; vm_init(&m);
    // variables:
#define native_function(name) \
    value_native(&val, hanayo::name);  env_set(m.env, #name, &val);
#define native_function_key(name, key) \
    value_native(&val, hanayo::name);  env_set(m.env, key, &val);
    struct value val;
    native_function(print)
    native_function(string)
    native_function_key(int_, "int")
    native_function_key(float_, "float")
    native_function(record)
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
