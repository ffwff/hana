#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include "array.h"
#include "value.h"
#include "map.h"

enum vm_opcode {
    OP_HALT,
    // stack manip
    OP_PUSH8, OP_PUSH16, OP_PUSH32, OP_PUSH64,
    OP_PUSHSTR,
    OP_POP,
    // arith
    OP_ADD, OP_SUB, OP_MUL, OP_DIV, OP_MOD,
    // logic
    OP_AND, OP_OR, OP_NOT,
    // comparison
    OP_LT, OP_LEQ, OP_GT, OP_GEQ,
    OP_EQ, OP_NEQ,
    // variables
    OP_SET, OP_GET,
    // flow control
    OP_JMP, OP_JCOND, OP_JNCOND, OP_CALL
};

typedef array(uint8_t) a_uint8;
typedef array(struct value) a_value;

struct vm {
    uint64_t ip;
    struct map env;
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
void vm_code_pushstr(struct vm *vm, const char *);

#ifdef __cplusplus
}
#endif
