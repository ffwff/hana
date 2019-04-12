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
            written += printf("%d", val->as.integer);
        else if(val->type == value::TYPE_FLOAT)
            written += printf("%f", val->as.floatp);
        value_free(val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
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
#ifndef RELEASE
    ast->print();
#endif

    // virtual machine
    struct vm m; vm_init(&m);
    // variables:
    struct value val;
    value_native(&val, hanayo::print);
    env_set(m.env, "print", &val);
    ast->emit(&m); // generate bytecodes
    array_push(m.code, OP_HALT);
    //while(vm_step(&m));
        //std::cin.get();
    vm_execute(&m);

    // cleanup
    vm_free(&m);
    return 0;
}
