#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include "array.h"
#include "value.h"
#include "env.h"

enum vm_opcode {
    OP_HALT,
    // stack manip
    OP_PUSH8, OP_PUSH16, OP_PUSH32, OP_PUSH64,
    OP_PUSH_NIL, OP_PUSHSTR, OP_PUSHF64,
    OP_POP,
    // arith
    OP_ADD, OP_SUB, OP_MUL, OP_DIV, OP_MOD,
    // logic
    OP_AND, OP_OR,
    // unary
    OP_NEGATE, OP_NOT,
    // comparison
    OP_LT, OP_LEQ, OP_GT, OP_GEQ,
    OP_EQ, OP_NEQ,
    // matching
    OP_OF /* type matching */,
    // variables
    OP_ENV_NEW,
    OP_SET_LOCAL, OP_SET_LOCAL_FUNCTION_DEF, OP_GET_LOCAL,
    OP_GET_LOCAL_UP,
    OP_SET_GLOBAL, OP_GET_GLOBAL,
    OP_DEF_FUNCTION_PUSH,
    // flow control
    OP_JMP, OP_JMP_LONG, OP_JCOND, OP_JNCOND, OP_CALL, OP_RET,
    // dictionary
    OP_DICT_NEW,
    OP_MEMBER_GET, OP_MEMBER_GET_NO_POP,
    OP_MEMBER_SET, OP_DICT_LOAD, OP_ARRAY_LOAD,
    OP_INDEX_GET, OP_INDEX_SET,
    // exceptions
    OP_TRY, OP_RAISE, OP_EXFRAME_RET,
    // tail calls
    OP_RETCALL,
    // iterators
    OP_FOR_IN, OP_SWAP,
    // modules
    OP_USE,
};

enum vm_error {
    ERROR_NO_ERROR = 0,
    ERROR_OP_ADD,
    ERROR_OP_SUB,
    ERROR_OP_MUL,
    ERROR_OP_DIV,
    ERROR_OP_MOD,
    ERROR_OP_AND,
    ERROR_OP_OR,
    ERROR_OP_LT,
    ERROR_OP_LEQ,
    ERROR_OP_GT,
    ERROR_OP_GEQ,
    ERROR_OP_EQ,
    ERROR_OP_NEQ,
    ERROR_UNDEFINED_GLOBAL_VAR,
    ERROR_RECORD_NO_CONSTRUCTOR,
    ERROR_CONSTRUCTOR_NOT_FUNCTION,
    ERROR_MISMATCH_ARGUMENTS,
    ERROR_EXPECTED_CALLABLE,
    ERROR_CANNOT_ACCESS_NON_RECORD,
    ERROR_KEY_NON_INT,
    ERROR_RECORD_KEY_NON_STRING,
    ERROR_UNBOUNDED_ACCESS,
    ERROR_EXPECTED_RECORD_ARRAY,
    ERROR_CASE_EXPECTS_DICT,
    ERROR_UNHANDLED_EXCEPTION,
    ERROR_EXPECTED_ITERABLE,
    ERROR_EXPECTED_RECORD_OF_EXPR,
};

typedef array(uint8_t) a_uint8;
typedef array(struct value) a_value;
struct exframe;
typedef array(struct exframe) a_exframe;
struct hmap;

struct vm {
    uint32_t ip;
    struct env *localenv, *localenv_bp;
    struct hmap *globalenv;
    a_exframe eframes;
    a_uint8 code;
    a_value stack;
    struct dict *dstr, *dint, *dfloat, *darray, *drec;
    enum vm_error error;
    uint32_t error_expected;

    struct exframe *exframe_fallthrough;
    size_t native_call_depth;
};

void vm_execute(struct vm*);
typedef array(struct value) a_arguments;

struct exframe *vm_enter_exframe(struct vm *);
bool vm_leave_exframe(struct vm *);
bool vm_raise(struct vm *);

struct value vm_call(struct vm *, const struct value, const a_arguments*);
struct env *vm_enter_env(struct vm *, struct function *);
struct env *vm_enter_env_tail(struct vm *, struct function *);
bool vm_leave_env(struct vm *);

void vm_load_module(struct vm*, const char*);

void vm_print_stack(const struct vm*);

void vm_code_push8(struct vm *vm, uint8_t);
void vm_code_pushstr(struct vm *vm, const char *);
void vm_code_pushf32(struct vm *vm, float f);
void vm_code_pushf64(struct vm *vm, double f);
void vm_code_fill(struct vm *vm, uint32_t, uint32_t);
void vm_code_fill16(struct vm *vm, uint32_t, uint16_t);

#ifdef __cplusplus
}
#endif
