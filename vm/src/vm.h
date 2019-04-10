#pragma once
#include <stdint.h>
#include "array.h"
#include "value.h"

enum vm_opcode {
    OP_HALT,
    OP_PUSH8, OP_PUSH16, OP_PUSH32, OP_PUSH64,
    OP_PUSHSTR,
    OP_POP,
    OP_ADD, OP_SUB, OP_MUL, OP_DIV, OP_MOD
};

typedef array(uint8_t) a_uint8;
typedef array(struct value) a_value;

struct vm {
    uint64_t ip;
    a_uint8 code;
    a_value stack;
};

void vm_init(struct vm*);
void vm_free(struct vm*);
int vm_step(struct vm*);
void vm_execute(struct vm*);
void vm_print_stack(const struct vm*);

void vm_code_push16(struct vm *vm, uint16_t);
void vm_code_push32(struct vm *vm, uint32_t);
void vm_code_push64(struct vm *vm, uint64_t);
void vm_code_pushstr(struct vm *vm, char *);
