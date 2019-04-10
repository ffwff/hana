#include <stdio.h>
#include <assert.h>
#include "vm.h"

// notes: architecture is big endian!

void vm_init(struct vm *vm) {
    vm->code = (a_uint8)array_init(uint8_t);
    vm->stack = (a_value)array_init(struct value);
    vm->ip = 0;
}

void vm_free(struct vm *vm) {
    array_free(vm->code);
    array_free(vm->stack);
}

int vm_step(struct vm *vm) {
    enum vm_opcode op = vm->code.data[vm->ip];
    if(op == OP_HALT) {
        printf("HALT\n");
        vm_print_stack(vm);
        return 0;
    }

    // stack manip
    else if (op == OP_PUSH8) {
        vm->ip++;
        const uint8_t data = vm->code.data[vm->ip++];
        printf("PUSH %d\n", data);

        array_push(vm->stack, (struct value){});
        value_int(&array_top(vm->stack), data);
    }
    else if (op == OP_PUSH16) {
        vm->ip++;
        //const uint16_t data =  << 8 | ;
        const uint8_t a = vm->code.data[vm->ip++],
                      b = vm->code.data[vm->ip++];
        printf("%x %x\n", a,b);
        const uint16_t data = a << 8 | b;

        array_push(vm->stack, (struct value){});
        value_int(&array_top(vm->stack), data);
    }

    else if (op == OP_POP) {
        printf("POP\n");
        assert(vm->stack.length > 0);
        vm->ip++;
        value_free(&array_top(vm->stack));
        vm->stack.length--;
    }

    // arith
#define arith_op(optype, fn) \
    else if (op == OP_ADD) { \
        printf("" #optype "\n"); \
        assert(vm->stack.length >= 2); \
        vm->ip++; \
\
        struct value left, right; \
        value_copy(&right, &array_top(vm->stack)); \
        array_pop(vm->stack); \
        value_copy(&left, &array_top(vm->stack)); \
        array_pop(vm->stack); \
\
        array_push(vm->stack, (struct value){}); \
        struct value *result = &array_top(vm->stack); \
        fn(result, &left, &right); \
    }
    arith_op(OP_ADD, value_add)
    arith_op(OP_SUB, value_sub)
    arith_op(OP_MUL, value_mul)
    arith_op(OP_DIV, value_div)
    arith_op(OP_MOD, value_mod)

    else {
        printf("undefined opcode: %d", op);
        assert(0);
    }

    vm_print_stack(vm);
    return 1;
}

void vm_execute(struct vm *vm) {
    while(vm_step(vm));
}

void vm_print_stack(const struct vm *vm) {
    printf("[");
    for(size_t i = 0; i < vm->stack.length; i++) {
        value_print(&vm->stack.data[i]);
        printf(" ");
    }
    printf("]\n");
}

// push bits
void vm_code_push16(struct vm *vm, uint16_t n) {
    printf("%x %x\n", (uint8_t)(n >> 8),n & 0x00ff);
    array_push(vm->code, (uint8_t)(n >> 8));
    array_push(vm->code, (uint8_t)(n & 0x00ff));
}

void vm_code_push32(struct vm *vm, uint32_t n) {
    array_push(vm->code, n & 0xff000000);
    array_push(vm->code, n & 0x00ff0000);
    array_push(vm->code, n & 0x0000ff00);
    array_push(vm->code, n & 0x000000ff);
}
