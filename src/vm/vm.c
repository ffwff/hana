#include <stdio.h>
#include <string.h>
#include "vm.h"
#include "string_.h"
#include "hmap.h"
#include "dict.h"
#include "array_obj.h"
#include "function.h"
#include "exception_frame.h"

#include <assert.h>
#ifdef DEBUG
#define debug_assert assert
#else
#define debug_assert(...)
#endif

#ifdef NOLOG
#define LOG(...)
#else
#define LOG(...) fprintf(stderr, __VA_ARGS__)
#endif
#define FATAL(...) fprintf(stderr, __VA_ARGS__)

void vm_execute(struct vm *vm) {
#define ERROR(code, unwind)           \
    do {                              \
        vm->error = code;             \
        vm->ip -= (uint32_t)(unwind); \
        return;                       \
    } while (0)
#define ERROR_EXPECT(code, unwind, expect)     \
    do {                                       \
        vm->error = code;                      \
        vm->ip -= (uint32_t)(unwind);          \
        vm->error_expected = (uint32_t)expect; \
        return;                                \
    } while (0)
#define doop(op) do_ ## op
#define X(op) [op] = && doop(op)
#ifdef NOLOG
#define dispatch()                                   \
    do {                                             \
        debug_assert(vm->ip <= vm->code.length);     \
        goto *dispatch_table[vm->code.data[vm->ip]]; \
    } while (0)
#else
#define dispatch()                                   \
    do {                                             \
        vm_print_stack(vm);                          \
        debug_assert(vm->ip <= vm->code.length);     \
        goto *dispatch_table[vm->code.data[vm->ip]]; \
    } while (0)
#endif

    static const void *dispatch_table[] = {
        X(OP_HALT),
        // stack manip
        X(OP_PUSH8), X(OP_PUSH16), X(OP_PUSH32), X(OP_PUSH64),
        X(OP_PUSH_NIL), X(OP_PUSHSTR), X(OP_PUSHF64),
        X(OP_POP),
        // arith
        X(OP_ADD), X(OP_SUB), X(OP_MUL), X(OP_DIV), X(OP_MOD),
        X(OP_IADD), X(OP_IMUL),
        // bitwise
        X(OP_BITWISE_AND), X(OP_BITWISE_OR), X(OP_BITWISE_XOR),
        // unary
        X(OP_NEGATE), X(OP_NOT),
        // comparison
        X(OP_LT), X(OP_LEQ), X(OP_GT), X(OP_GEQ),
        X(OP_EQ), X(OP_NEQ),
        // matching
        X(OP_OF) /* type matching */,
        // variables
        X(OP_ENV_NEW),
        X(OP_SET_LOCAL), X(OP_SET_LOCAL_FUNCTION_DEF), X(OP_GET_LOCAL),
        X(OP_GET_LOCAL_UP),
        X(OP_SET_GLOBAL), X(OP_GET_GLOBAL),
        X(OP_DEF_FUNCTION_PUSH),
        // flow control
        X(OP_JMP), X(OP_JMP_LONG), X(OP_JCOND), X(OP_JNCOND), X(OP_CALL), X(OP_RET),
        X(OP_JCOND_NO_POP), X(OP_JNCOND_NO_POP),
        // record
        X(OP_DICT_NEW),
        X(OP_MEMBER_GET), X(OP_MEMBER_GET_NO_POP),
        X(OP_MEMBER_SET), X(OP_DICT_LOAD), X(OP_ARRAY_LOAD),
        X(OP_INDEX_GET), X(OP_INDEX_GET_NO_POP), X(OP_INDEX_SET),
        // exceptions
        X(OP_TRY), X(OP_RAISE), X(OP_EXFRAME_RET),
        // tail calls
        X(OP_RETCALL),
        // iterators
        X(OP_FOR_IN), X(OP_SWAP),
        // modules
        X(OP_USE)};

#undef X

    dispatch();

    // halt
    doop(OP_HALT): {
        LOG("HALT\n");
        return;
    }

    // stack manip
    // push uint family on to the stack
#define push_int_op(optype, _type, _data)                           \
    doop(optype) : {                                                \
        vm->ip++;                                                   \
        const _type data = _data;                                   \
        vm->ip += (uint32_t)sizeof(_type);                          \
        LOG(sizeof(_type) == 8 ? "PUSH %ld\n" : "PUSH %d\n", data); \
        array_push(vm->stack, value_int((int64_t)data));            \
        dispatch();                                                 \
    }
    push_int_op(OP_PUSH8,  uint8_t,  vm->code.data[vm->ip+0])

    push_int_op(OP_PUSH16, uint16_t, (uint16_t)(vm->code.data[vm->ip+0] << 8 |
                                               vm->code.data[vm->ip+1]))

    push_int_op(OP_PUSH32, uint32_t, (uint32_t)(vm->code.data[vm->ip+0] << 24 |
                                                vm->code.data[vm->ip+1] << 16 |
                                                vm->code.data[vm->ip+2] << 8  |
                                                vm->code.data[vm->ip+3]))

    push_int_op(OP_PUSH64, uint64_t, (uint64_t)vm->code.data[vm->ip+0] << 56 |
                                     (uint64_t)vm->code.data[vm->ip+1] << 48 |
                                     (uint64_t)vm->code.data[vm->ip+2] << 40 |
                                     (uint64_t)vm->code.data[vm->ip+3] << 32 |
                                     (uint64_t)vm->code.data[vm->ip+4] << 24 |
                                     (uint64_t)vm->code.data[vm->ip+5] << 16 |
                                     (uint64_t)vm->code.data[vm->ip+6] << 8  |
                                     (uint64_t)vm->code.data[vm->ip+7])

    // push 32/64-bit float on to the stack
    doop(OP_PUSHF64): {
        vm->ip++;
        union {
            double d;
            uint8_t u[4];
        } u;
        u.u[0] = vm->code.data[vm->ip + 0];
        u.u[1] = vm->code.data[vm->ip + 1];
        u.u[2] = vm->code.data[vm->ip + 2];
        u.u[3] = vm->code.data[vm->ip + 3];
        u.u[4] = vm->code.data[vm->ip + 4];
        u.u[5] = vm->code.data[vm->ip + 5];
        u.u[6] = vm->code.data[vm->ip + 6];
        u.u[7] = vm->code.data[vm->ip + 7];
        vm->ip += (uint32_t)sizeof(u);
        LOG("PUSH_F64 %f\n", u.d);
        array_push(vm->stack, value_float(u.d));
        dispatch();
    }

    // push string on to the stack
    doop(OP_PUSHSTR): {
        vm->ip++;
        char *str = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += (uint32_t)strlen(str) + 1;
        LOG("PUSH %s\n", str);
        array_push(vm->stack, value_str(str, vm));
        dispatch();
    }

    // push nil to the stack
    doop(OP_PUSH_NIL): {
        vm->ip++;
        LOG("PUSH NIL\n");
        array_push(vm->stack, value_pointer(TYPE_NIL, 0));
        dispatch();
    }

    // frees top of the stack and pops the stack
    doop(OP_POP): {
        LOG("POP\n");
        vm->ip++;
        array_pop(vm->stack);
        dispatch();
    }

    // pops top of the stack, performs unary not and pushes the result
    doop(OP_NOT): {
        vm->ip++;
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        array_push(vm->stack, value_int(!value_is_true(val)));
        dispatch();
    }
    // pops top of the stack, performs unary negation and pushes the result
    doop(OP_NEGATE): {
        vm->ip++;
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        switch (val.type) {
            case TYPE_INT: {
                array_push(vm->stack, value_int(-value_get_int(val)));
                break;
            }
            case TYPE_FLOAT: {
                array_push(vm->stack, value_float(-value_get_float(val)));
                break;
            }
            default: assert(0);
        }
        dispatch();
    }

    // binary ops: perform binary operations on the 2 top values of the stack
    // arithmetic:
#define binop(optype, fn)                                          \
    doop(optype) : {                                               \
        vm->ip++;                                                  \
        LOG(#optype "\n");                                         \
        debug_assert(vm->stack.length >= 2);                       \
                                                                   \
        struct value right = vm->stack.data[vm->stack.length - 1]; \
        struct value left = vm->stack.data[vm->stack.length - 2];  \
                                                                   \
        struct value result = fn(left, right, vm);                 \
        if (result.type == TYPE_INTERPRETER_ERROR) {    \
            ERROR(ERROR_##optype, 1);                              \
        }                                                          \
        vm->stack.length -= 2;                                     \
        array_push(vm->stack, result);                             \
        dispatch();                                                \
    }
    binop(OP_ADD, value_add)
    binop(OP_SUB, value_sub)
    binop(OP_MUL, value_mul)
    binop(OP_DIV, value_div)
    binop(OP_MOD, value_mod)

    binop(OP_BITWISE_AND, value_bitwise_and)
    binop(OP_BITWISE_OR, value_bitwise_or)
    binop(OP_BITWISE_XOR, value_bitwise_xor)

    // in place arithmetic:
    // does regular arith, returns lhs on stack and jumps out of fallback if CAN do it in place (for primitives)
    // else just does the fallback (copying and setting variable manually)
#define binop_inplace(optype, errortype, fn, fallback)               \
    doop(optype) : {                                                 \
        vm->ip++;                                                    \
        LOG(#optype "\n");                                           \
        debug_assert(vm->stack.length >= 2);                         \
        const uint8_t pos = (uint8_t)vm->code.data[vm->ip];          \
                                                                     \
        struct value right = vm->stack.data[vm->stack.length - 1];   \
        struct value left = vm->stack.data[vm->stack.length - 2];    \
        if (fn(left, right)) {                                       \
            LOG("did in place!\n");                                  \
            vm->stack.length--;                                      \
            vm->ip += pos;                                           \
            dispatch();                                              \
        }                                                            \
        struct value result = fallback(left, right, vm);             \
        if (result.type == TYPE_INTERPRETER_ERROR) {      \
            ERROR(errortype, 1);                                     \
        }                                                            \
        vm->stack.length -= 2;                                       \
        array_push(vm->stack, result);                               \
        vm->ip++;                                                    \
        dispatch();                                                  \
    }

    binop_inplace(OP_IADD, ERROR_OP_ADD, value_iadd, value_add)
    binop_inplace(OP_IMUL, ERROR_OP_MUL, value_imul, value_mul)

    // comparison
    binop(OP_LT,  value_lt)
    binop(OP_LEQ, value_leq)
    binop(OP_GT,  value_gt)
    binop(OP_GEQ, value_geq)
    binop(OP_EQ,  value_eq)
    binop(OP_NEQ, value_neq)

    // matching (these require the stdlib to be loaded)
    doop(OP_OF): {
        LOG("OF\n");
        debug_assert(vm->stack.length >= 2);
        vm->ip++;

        struct value right = array_top(vm->stack);
        array_pop(vm->stack);
        struct value left = array_top(vm->stack);
        array_pop(vm->stack);

        if (right.type == TYPE_DICT) {
            const struct dict *rhs = value_get_pointer(right);
            if (left.type == TYPE_DICT) {
                if (rhs == vm->drec) {
                    array_push(vm->stack, value_int(1));
                } else {
                    array_push(vm->stack, value_int(dict_is_prototype_of(value_get_pointer(left), rhs)));
                }
            } else if (value_get_prototype(vm, left) == rhs) {
                array_push(vm->stack, value_int(1));
            } else {
                array_push(vm->stack, value_int(0));
            }
        } else {
            ERROR(ERROR_EXPECTED_RECORD_OF_EXPR, 1);
        }

        dispatch();
    }

    // variables
    // creates a new environment whenever a function is called
    // the environment is initialized with a copy of the current environment's variables
    doop(OP_ENV_NEW): {
        vm->ip++;
        const uint16_t n = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(n);
        LOG("RESERVE %d\n", n);
        env_init(vm->localenv, n, vm);
        dispatch();
    }

    // variables
    // sets the value of current environment's slot to the top of the stack
    doop(OP_SET_LOCAL): {
        vm->ip++;
        const uint16_t n = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(n);
        LOG("SET LOCAL %d\n", n);
        env_set(vm->localenv, n, array_top(vm->stack));
        dispatch();
    }
    // this is for recursive function
    doop(OP_SET_LOCAL_FUNCTION_DEF): {
        vm->ip++;
        const uint16_t n = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(n);
        struct value val = array_top(vm->stack);
        env_set(vm->localenv, n, val);
        function_set_bound_var(value_get_pointer(val), n, val);
        dispatch();
    }
    // pushes a copy of the value of current environment's slot
    doop(OP_GET_LOCAL): {
        vm->ip++;
        const uint16_t n = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(n);
        LOG("GET LOCAL %d\n", n);
        array_push(vm->stack, (struct value){0});
        array_top(vm->stack) = env_get(vm->localenv, n);
        dispatch();
    }
    doop(OP_GET_LOCAL_UP): {
        vm->ip++;
        const uint16_t n = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(n);
        const uint16_t relascope = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                              vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(relascope);
        LOG("GET LOCAL UP %d %d\n", n, relascope);

        array_push(vm->stack, (struct value){0});
        array_top(vm->stack) = env_get_up(vm->localenv, relascope, n);
        dispatch();
    }

    // sets the value of the global variable to the top of the stack
    doop(OP_SET_GLOBAL): {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += (uint32_t)strlen(key) + 1;
        LOG("SET GLOBAL %s %p\n", key, vm->globalenv);
        hmap_set(vm->globalenv, key, array_top(vm->stack));
        dispatch();
    }
    // pushes a copy of the value of the global variable
    doop(OP_GET_GLOBAL): {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += (uint32_t)strlen(key) + 1;
        LOG("GET GLOBAL %s\n", key);
        const struct value *val = hmap_get(vm->globalenv, key);
        if(val == NULL) {
            ERROR(ERROR_UNDEFINED_GLOBAL_VAR, 1 + strlen(key)+1);
        } else {
            array_push(vm->stack, *val);
        }
        dispatch();
    }

    // pushes a function with [name], that begins at the next instruction pointer
    // to the stack and jumps to the [end address]
    doop(OP_DEF_FUNCTION_PUSH): {
        // [opcode][end address]
        vm->ip++;
        const uint16_t nargs = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                          vm->code.data[vm->ip + 1]);
        vm->ip += (uint32_t)sizeof(nargs);
        const uint16_t pos = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                        vm->code.data[vm->ip + 1]);
        LOG("DEF_FUNCTION_PUSH %d %d\n", pos, nargs);
        array_push(vm->stack, value_function(vm->ip + (uint32_t)sizeof(pos), nargs, vm->localenv, vm));

        vm->ip += pos;
        dispatch();
    }

    // flow control
    doop(OP_JMP): { // jmp [32-bit position]
        vm->ip++;
        const int16_t pos = (int16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        vm->ip += pos;
        LOG("JMP %d\n", pos);
        dispatch();
    }
    doop(OP_JMP_LONG): { // jmp [32-bit position]
        vm->ip++;
        const uint32_t pos = (uint32_t)(vm->code.data[vm->ip + 0] << 24 |
                                        vm->code.data[vm->ip + 1] << 16 |
                                        vm->code.data[vm->ip + 2] << 8 |
                                        vm->code.data[vm->ip + 3]);
        vm->ip = pos;
        LOG("JMP LONG %d\n", pos);
        dispatch();
    }
    doop(OP_JCOND):
    doop(OP_JCOND_NO_POP): { // jmp if not true [32-bit position]
        const enum vm_opcode op = vm->code.data[vm->ip++];
        const int16_t pos = (int16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        struct value val = array_top(vm->stack);
        if(op == OP_JCOND) array_pop(vm->stack);
        LOG(op == OP_JCOND ? "JCOND %d\n" : "JCOND_NO_POP %d\n", pos);
        if(value_is_true(val)) vm->ip += pos;
        else vm->ip += (uint32_t)sizeof(pos);
        dispatch();
    }
    doop(OP_JNCOND):
    doop(OP_JNCOND_NO_POP):  { // jump if true [32-bit position]
        const enum vm_opcode op = vm->code.data[vm->ip++];
        const int16_t pos = (int16_t)(vm->code.data[vm->ip + 0] << 8 |
                                      vm->code.data[vm->ip + 1]);
        struct value val = array_top(vm->stack);
        if(op == OP_JNCOND) array_pop(vm->stack);
        LOG(op == OP_JNCOND ? "JNCOND %d\n" : "JNCOND_NO_POP %d\n", pos);
        if(!value_is_true(val)) vm->ip += pos;
        else vm->ip += (uint32_t)sizeof(pos);
        dispatch();
    }
    // pops a function/record constructor on top of the stack,
    // sets up necessary environment and calls it.
#define CALL_NATIVE(expr)                                                                     \
    do {                                                                                      \
        vm->native_call_depth++;                                                              \
        expr(vm, nargs);                                                                      \
        vm->native_call_depth--;                                                              \
        if ((vm->exframe_fallthrough != NULL &&                                               \
             exframe_native_stack_depth(vm->exframe_fallthrough) != vm->native_call_depth) || \
            vm->error != ERROR_NO_ERROR)                                                      \
            return;                                                                           \
    } while (0)
#define JMP_INTERPRETED_FN_(POP, UNWIND, END_IF_NATIVE)                                             \
    do {                                                                                            \
        if (val.type == TYPE_DICT) {                                                     \
            do {                                                                                    \
                POP                                                                                 \
            } while (0);                                                                            \
            const struct value *pctor = dict_get(value_get_pointer(val), "constructor"); \
            if (pctor == NULL) {                                                                    \
                ERROR(ERROR_RECORD_NO_CONSTRUCTOR, UNWIND);                                         \
            }                                                                                       \
            const struct value ctor = *pctor;                                                       \
            switch (ctor.type) {                                                         \
                case TYPE_NATIVE_FN: {                                                              \
                    CALL_NATIVE(((value_fn)value_get_pointer(ctor)));               \
                    do {                                                                            \
                        END_IF_NATIVE                                                               \
                    } while (0);                                                                    \
                    break;                                                                          \
                }                                                                                   \
                case TYPE_FN: {                                                                     \
                    ifn = value_get_pointer(ctor);                                         \
                    if (nargs + 1 != ifn->nargs) {                                                  \
                        ERROR_EXPECT(ERROR_MISMATCH_ARGUMENTS, ifn->nargs, UNWIND);                 \
                    }                                                                               \
                    struct value new_val = value_dict(vm);                                          \
                    dict_set(value_get_pointer(new_val), "prototype", val);              \
                    array_push(vm->stack, new_val);                                                 \
                    break;                                                                          \
                }                                                                                   \
                default:                                                                            \
                    ERROR(ERROR_CONSTRUCTOR_NOT_FUNCTION, UNWIND);                                  \
            }                                                                                       \
        } else {                                                                                    \
            do {                                                                                    \
                POP                                                                                 \
            } while (0);                                                                            \
            ifn = value_get_pointer(val);                                                  \
            if (nargs != ifn->nargs) {                                                              \
                ERROR_EXPECT(ERROR_MISMATCH_ARGUMENTS, ifn->nargs, UNWIND);                         \
            }                                                                                       \
        }                                                                                           \
    } while (0)
#define JMP_INTERPRETED_FN(UNWIND, END_IF_NATIVE) JMP_INTERPRETED_FN_(array_pop(vm->stack);, UNWIND, END_IF_NATIVE)
#define JMP_INTERPRETED_FN_NO_POP(UNWIND, END_IF_NATIVE) JMP_INTERPRETED_FN_(do{}while(0);, UNWIND, END_IF_NATIVE)
    doop(OP_CALL) : {
        // argument: [arg2][arg1]
        vm->ip++;
        struct value val = array_top(vm->stack);
        const uint16_t nargs = (uint16_t)(vm->code.data[vm->ip+0] << 8 |
                                          vm->code.data[vm->ip+1]);
        vm->ip += (uint32_t)sizeof(nargs);
        debug_assert(vm->stack.length >= nargs);
        LOG("call %d\n", nargs);
        switch(val.type) {
        case TYPE_NATIVE_FN: {
            array_pop(vm->stack);
            CALL_NATIVE(((value_fn)(value_get_pointer(val))));
            break; }
        case TYPE_FN:
        case TYPE_DICT: {
            struct function *ifn;
            JMP_INTERPRETED_FN(1 + sizeof(nargs), {
                if (vm->exframe_fallthrough != NULL) {
                    if (exframe_native_stack_depth(vm->exframe_fallthrough) == vm->native_call_depth) {
                        dispatch();
                    } else {
                        return;
                    }
                }
                dispatch();
            });

            // caller
            vm_enter_env(vm, ifn);
            break; }
        default: {
            ERROR(ERROR_EXPECTED_CALLABLE, 1 + sizeof(nargs)); }
        }
        dispatch();
    }
    // returns from function
    doop(OP_RET): {
        LOG("RET %p\n", vm->localenv);
        if (vm_leave_env(vm)) {
            LOG("return from vm_call\n");
            return;
        }
        LOG("ip = %d\n", vm->ip);
        dispatch();
    }

    // dictionaries
    doop(OP_DICT_NEW): {
        vm->ip++;
        assert(0);
    }
    doop(OP_MEMBER_GET):
    doop(OP_MEMBER_GET_NO_POP): {
        const enum vm_opcode op = vm->code.data[vm->ip];
        vm->ip++;
        const char *key = (const char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += (uint32_t)strlen(key) + 1;
        LOG(op == OP_MEMBER_GET ? "MEMBER_GET %s\n" : "MEMBER_GET_NO_POP %s\n", key);

        const struct value val = array_top(vm->stack);
        struct dict *dict = NULL;
        if(val.type != TYPE_DICT) {
            if((dict = value_get_prototype(vm, val)) == NULL) {
                ERROR(ERROR_CANNOT_ACCESS_NON_RECORD, 1 + strlen(key)+1);
            }
            if (strcmp(key, "prototype") == 0) {
                if (op == OP_MEMBER_GET) array_pop(vm->stack);
                array_push(vm->stack, value_pointer(TYPE_DICT, dict));
                dispatch();
            }
        } else {
            dict = value_get_pointer(val);
            if(op == OP_MEMBER_GET) array_pop(vm->stack);
        }

        const struct value *result = dict_get(dict, key);
        if(result != NULL) {
            array_push(vm->stack, *result);
        } else {
            ERROR(ERROR_UNKNOWN_KEY, strlen(key)+1);
        }
        dispatch();
    }
    doop(OP_MEMBER_SET): {
        // stack: [value][dict]
        vm->ip++;
        const char *key = (char *)(vm->code.data + vm->ip);  // must be null terminated
        vm->ip += (uint32_t)strlen(key) + 1;
        LOG("MEMBER_SET %s\n", key);
        const struct value dval = array_top(vm->stack);
        if(dval.type != TYPE_DICT) {
            ERROR(ERROR_CANNOT_ACCESS_NON_RECORD, 1 + strlen(key)+1);
        }
        array_pop(vm->stack);

        struct value val = array_top(vm->stack);
        dict_set(value_get_pointer(dval), key, val);
        dispatch();
    }
    doop(OP_DICT_LOAD): {
        // stack: [nil][value][key]
        vm->ip++;

        struct value val = array_top(vm->stack);
        size_t length = (size_t)value_get_int(val);
        array_pop(vm->stack);
        debug_assert(val.type == TYPE_INT);
        LOG("DICT_LOAD %ld\n", length);

        struct value dval = value_dict_n(length, vm);
        struct dict *dict = value_get_pointer(dval);

        while(length--) {
            debug_assert(key.type == TYPE_STR);
            // key
            struct value key = array_top(vm->stack);
            array_pop(vm->stack);
            // val
            struct value val = array_top(vm->stack);
            array_pop(vm->stack);
            dict_set_str(dict, value_get_pointer(key), val);
        }

        array_push(vm->stack, dval);
        dispatch();
    }
    // array
    doop(OP_INDEX_GET):
    doop(OP_INDEX_GET_NO_POP): {
        const enum vm_opcode op = vm->code.data[vm->ip];
        vm->ip++;
        LOG(op == OP_INDEX_GET ? "INDEX_GET\n" : "INDEX_GET_NO_POP\n");

        const struct value index = array_top(vm->stack);
        array_pop(vm->stack);

        const struct value dval = array_top(vm->stack);
        if (op == OP_INDEX_GET) array_pop(vm->stack);

        switch (dval.type) {
            case TYPE_ARRAY: {
                array_obj *array = value_get_pointer(dval);
                if (index.type != TYPE_INT) {
                    ERROR(ERROR_KEY_NON_INT, 1);
                }
                const int32_t i = (int32_t)value_get_int(index);
                if (!(i >= 0 && i < (int32_t)array->length)) {
                    ERROR_EXPECT(ERROR_UNBOUNDED_ACCESS, 1, array->length);
                }
                array_push(vm->stack, array->data[i]);
                break;
            }
            case TYPE_STR: {
                if (index.type != TYPE_INT) {
                    ERROR(ERROR_KEY_NON_INT, 1);
                }
                const int32_t i = (int32_t)value_get_int(index);
                struct string *s = string_at(value_get_pointer(dval), i, vm);
                if (s == NULL) {
                    ERROR_EXPECT(ERROR_UNBOUNDED_ACCESS, 1, string_len(value_get_pointer(dval)));
                }
                array_push(vm->stack, value_pointer(TYPE_STR, s));
                break;
            }
            case TYPE_DICT: {
                if (index.type != TYPE_STR) {
                    ERROR(ERROR_RECORD_KEY_NON_STRING, 1);
                }
                const struct value *val = dict_get_str(value_get_pointer(dval), value_get_pointer(index));
                if (val != NULL) {
                    array_push(vm->stack, *val);
                } else {
                    ERROR(ERROR_UNKNOWN_KEY, 1);
                }
                break;
            }
            default:
                ERROR(ERROR_CANNOT_ACCESS_NON_RECORD, 1);
        }
        dispatch();
    }
    doop(OP_INDEX_SET): {
        vm->ip++;
        LOG("INDEX_SET\n");

        const struct value index = array_top(vm->stack);
        array_pop(vm->stack);

        const struct value dval = array_top(vm->stack);
        array_pop(vm->stack);

        const struct value val = array_top(vm->stack);
        switch (dval.type) {
            case TYPE_ARRAY: {
                if (index.type != TYPE_INT) {
                    ERROR(ERROR_KEY_NON_INT, 1);
                }
                const int64_t i = value_get_int(index);
                array_obj *array = value_get_pointer(dval);
                if (!(i >= 0 && i < (int64_t)array->length)) {
                    ERROR_EXPECT(ERROR_UNBOUNDED_ACCESS, array->length, 1);
                }
                array->data[i] = val;
                break;
            }
            case TYPE_DICT: {
                if (index.type != TYPE_STR) {
                    ERROR(ERROR_RECORD_KEY_NON_STRING, 1);
                }
                dict_set_str(value_get_pointer(dval), value_get_pointer(index), val);
                break;
            }
            default:
                ERROR(ERROR_EXPECTED_RECORD_ARRAY, 1);
        }
        dispatch();
    }
    doop(OP_ARRAY_LOAD) : {
        vm->ip++;

        size_t length = (size_t)value_get_int(array_top(vm->stack));
        array_pop(vm->stack);
        LOG("ARRAY_LOAD %ld\n", length);

        if(length == 0) {
            array_push(vm->stack, value_array(vm));
        } else {
            struct value aval = value_array_n(length, vm);
            array_obj *array = value_get_pointer(aval);
            array->length = length;
            while(length--) {
                array->data[length] = array_top(vm->stack);
                array_pop(vm->stack);
            }
            array_push(vm->stack, aval);
        }
        dispatch();
    }

    // exceptions
    doop(OP_TRY): {
        // stack: [nil][function][error type]
        LOG("TRY\n");
        vm->ip++;

        struct exframe *frame = vm_enter_exframe(vm);
        struct value error = {0};
        while((error = array_top(vm->stack)).type != TYPE_NIL) {
            // error type
            if(error.type != TYPE_DICT) {
                ERROR(ERROR_CASE_EXPECTS_DICT, 1);
            }
            array_pop(vm->stack);
            // val
            struct value fn = array_top(vm->stack);
            debug_assert(fn.type == TYPE_FN);
            array_pop(vm->stack);
            exframe_set_handler(frame, value_get_pointer(error), value_get_pointer(fn));
        }
        array_pop(vm->stack); // pop nil

        dispatch();
    }
    doop(OP_RAISE): {
        LOG("RAISE\n");
        if(!vm_raise(vm)) {
            vm->error = ERROR_UNHANDLED_EXCEPTION;
            if (vm->exframe_fallthrough != NULL || vm->native_call_depth != 0) {
                LOG("falling through pls wait (%ld)\n", vm->native_call_depth);
                return;
            }
            return;
        }
        if (vm->exframe_fallthrough != NULL || vm->native_call_depth != 0) {
            LOG("falling through pls wait (%ld)\n", vm->native_call_depth);
            return;
        }
        dispatch();
    }
    doop(OP_EXFRAME_RET): {
        vm->ip++;
        const uint16_t pos = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                        vm->code.data[vm->ip + 1]);
        vm->ip += pos;
        LOG("EXFRAME_RET %d\n", pos);
        vm_leave_exframe(vm);
        dispatch();
    }

    // tail calls
    doop(OP_RETCALL): {
        vm->ip++;
        struct value val = array_top(vm->stack);
        const uint16_t nargs = (uint16_t)(vm->code.data[vm->ip+0] << 8 |
                                          vm->code.data[vm->ip+1]);
        vm->ip += (uint32_t)sizeof(nargs);
        debug_assert(vm->stack.length >= nargs);
        LOG("retcall %d\n", nargs);
        switch (val.type) {
            case TYPE_NATIVE_FN: {
                array_pop(vm->stack);
                CALL_NATIVE(((value_fn)value_get_pointer(val)));
                if (vm_leave_env(vm)) {
                    LOG("return from vm_call\n");
                    return;
                }
                break;
            }
            case TYPE_FN:
            case TYPE_DICT: {
                struct function *ifn;
                JMP_INTERPRETED_FN(1 + sizeof(nargs),
                                   if (vm_leave_env(vm)) {
                                       LOG("return from vm_call\n");
                                       return;
                                   } else {
                                       dispatch();
                                   });
                // caller
                vm_enter_env_tail(vm, ifn);
                break;
            }
            default: {
                ERROR(ERROR_EXPECTED_CALLABLE, 1 + sizeof(nargs));
            }
        }
        dispatch();
    }

    // operators
    doop(OP_FOR_IN): {
#define CALL_DICT_ITERATOR_FN                                                                               \
    do {                                                                                                    \
        if (pval == NULL) ERROR(ERROR_EXPECTED_CALLABLE, 1 + sizeof(pos));                                  \
        const struct value val = *pval;                                                                     \
        const uint16_t nargs = 1;                                                                           \
        switch (val.type) {                                                                      \
            case TYPE_NATIVE_FN: {                                                                          \
                CALL_NATIVE(((value_fn)(value_get_pointer(val))));                          \
                break;                                                                                      \
            }                                                                                               \
            case TYPE_FN:                                                                                   \
            case TYPE_DICT: {                                                                               \
                struct function *ifn;                                                                       \
                JMP_INTERPRETED_FN_NO_POP(1 + sizeof(pos), {                                                \
                    if (vm->exframe_fallthrough != NULL) {                                                  \
                        if (exframe_native_stack_depth(vm->exframe_fallthrough) == vm->native_call_depth) { \
                            dispatch();                                                                     \
                        } else {                                                                            \
                            return;                                                                         \
                        }                                                                                   \
                    }                                                                                       \
                    dispatch();                                                                             \
                });                                                                                         \
                vm_enter_env(vm, ifn);                                                                      \
                dispatch();                                                                                 \
            }                                                                                               \
            default:                                                                                        \
                ERROR(ERROR_EXPECTED_CALLABLE, 1 + sizeof(pos));                                            \
        }                                                                                                   \
    } while (0);
        vm->ip++;
        const uint16_t pos = (uint16_t)(vm->code.data[vm->ip + 0] << 8 |
                                        vm->code.data[vm->ip + 1]);
        LOG("FOR_IN %d\n", pos);
        debug_assert(vm->stack.length > 0);
        struct value top = array_top(vm->stack);
        switch (top.type) {
            // setup
            case TYPE_STR: {
                array_obj *chars = string_chars(value_get_pointer(top), vm);
                array_pop(vm->stack);
                array_push(vm->stack, value_pointer(TYPE_ARRAY, chars));
                array_push(vm->stack, value_pointer(TYPE_INTERPRETER_ITERATOR, (void *)1));
                array_push(vm->stack, chars->data[0]);
                break;
            }
            case TYPE_ARRAY: {
                array_obj *array = value_get_pointer(top);
                if (array->length == 0) {  // skip empty
                    vm->ip += pos;
                    array_pop(vm->stack);
                    dispatch();
                }
                array_push(vm->stack, value_pointer(TYPE_INTERPRETER_ITERATOR, (void *)1));
                array_push(vm->stack, array->data[0]);
                break;
            }
            case TYPE_DICT: {
                struct dict *dict = value_get_pointer(top);
                const struct value *pval = dict_get(dict, "next");
                array_push(vm->stack, value_pointer(TYPE_INTERPRETER_ITERATOR, (void *)0));
                vm->ip += (uint32_t)sizeof(pos);
                array_push(vm->stack, top);  // pass arg
                CALL_DICT_ITERATOR_FN
                break;
            }
            // interation
            case TYPE_INTERPRETER_ITERATOR: {
                debug_assert(vm->stack.length >= 2);
                const struct value iterator = vm->stack.data[vm->stack.length - 2];
                switch (iterator.type) {
                    case TYPE_NIL: {
                        LOG("END\n");
                        vm->ip += pos;
                        dispatch();
                    }
                    case TYPE_ARRAY: {
                        const array_obj *array = value_get_pointer(iterator);
                        const size_t idx = (size_t)value_get_int(top);
                        if (idx == array->length) { /* end of it */
                            LOG("END\n");
                            array_pop(vm->stack);   /* iterator */
                            array_pop(vm->stack);   /* array */
                            vm->ip += pos;
                            dispatch();
                        } else {  // continuation
                            LOG("CONTINUE\n");
                            array_pop(vm->stack); /* old iterator */
                            array_push(vm->stack, value_pointer(TYPE_INTERPRETER_ITERATOR, (void *)(idx + 1)));
                            array_push(vm->stack, array->data[idx]);
                        }
                        break;
                    }
                    case TYPE_DICT: {
                        LOG("CONTINUE\n");
                        const struct dict *dict = value_get_pointer(iterator);
                        if (dict_get(dict, "stopped") != NULL) {
                            array_pop(vm->stack); /* iterator */
                            array_pop(vm->stack); /* record */
                            vm->ip += pos;
                            dispatch();
                        }

                        const struct value *pval = dict_get(dict, "next");
                        vm->ip += (uint32_t)sizeof(pos);
                        array_push(vm->stack, iterator);  // arg
                        CALL_DICT_ITERATOR_FN
                        break;
                    }
                    default:
                        ERROR(ERROR_EXPECTED_ITERABLE, 1 + sizeof(pos));
                }
                break;
            }
            default: {
                LOG("NOT ITERABLE\n");
                ERROR(ERROR_EXPECTED_ITERABLE, sizeof(pos));
            }
        }
        vm->ip += (uint32_t)sizeof(pos);
        dispatch();
#undef CALL_DICT_ITERATOR_FN
    }

    doop(OP_SWAP): {
        vm->ip++;
        debug_assert(vm->stack.length >= 2);
        const struct value lower = vm->stack.data[vm->stack.length - 2];
        const struct value higher = vm->stack.data[vm->stack.length - 1];
        vm->stack.data[vm->stack.length - 1] = lower;
        vm->stack.data[vm->stack.length - 2] = higher;
        dispatch();
    }

    // modules
    doop(OP_USE): {
        vm->ip++;
        char *str = (char *)&vm->code.data[vm->ip]; // must be null terminated
        LOG("USE %s\n", str);
        vm->ip += (uint32_t)strlen(str) + 1;
        vm_load_module(vm, str);
        dispatch();
    }
}

struct value vm_call(struct vm *vm, const struct value fn, const a_arguments *args) {
    struct function *ifn = NULL;
    const uint16_t nargs = args->length;

    if (fn.type == TYPE_NATIVE_FN) {
        for (size_t i = args->length; i-- > 0;) {
            array_push(vm->stack, args->data[i]);
        }
        ((value_fn)(value_get_pointer(fn)))(vm, nargs);
    } else if(fn.type == TYPE_DICT) {
        const struct value *pctor = dict_get(value_get_pointer(fn), "constructor");
        if(pctor == NULL) {
            vm->error = ERROR_RECORD_NO_CONSTRUCTOR;
            return value_interpreter_error();
        }
        const struct value ctor = *pctor;
        if(ctor.type == TYPE_NATIVE_FN) {
            for (size_t i = args->length; i-- > 0;) {
                array_push(vm->stack, args->data[i]);
            }
            ((value_fn)(value_get_pointer(ctor)))(vm, nargs);
            if (vm->error) return value_interpreter_error();
            const struct value val = array_top(vm->stack);
            array_pop(vm->stack);
            return val;
        } else if (ctor.type == TYPE_FN) {
            ifn = value_get_pointer(ctor);
        } else {
            vm->error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
            return value_interpreter_error();
        }
    } else if (fn.type == TYPE_FN) {
        ifn = value_get_pointer(fn);
    } else {
        vm->error = ERROR_EXPECTED_CALLABLE;
        return value_interpreter_error();
    }

    if((uint32_t)args->length != ifn->nargs) {
        vm->ip = ifn->ip;
        vm->error = ERROR_MISMATCH_ARGUMENTS;
        return value_interpreter_error();
    }

    const uint32_t last = vm->ip;
    // setup env
    struct env *oldenv = vm->localenv;
    vm->ip = (uint32_t)-1;
    struct env *curenv = vm_enter_env(vm, ifn);
    // setup stack/ip
    printf("%ld\n", args->length);
    for (size_t i = (size_t)args->length; i-- > 0;) {
        array_push(vm->stack, args->data[i]);
    }
    // call it
    vm_execute(vm);
    if(vm->error || vm->exframe_fallthrough != NULL) { // exception
        LOG("falling through psl wait %ld", vm->native_call_depth);
        return value_interpreter_error();
    }
    if(vm->localenv != curenv) { // exception occurred outside of function's scope
        // NOTE: curenv already free'd from unwinding
        return value_interpreter_error();
    }
    // restore ip
    LOG("vm_call complete\n");
    env_free(curenv);
    vm->localenv = oldenv;
    vm->ip = last;

    const struct value val = array_top(vm->stack);
    array_pop(vm->stack);
    return val;
}

void vm_print_stack(const struct vm *vm) {
    fprintf(stderr, "[");
    for(size_t i = 0; i < vm->stack.length; i++) {
        value_print(vm->stack.data[i]);
        fprintf(stderr, " ");
    }
    fprintf(stderr, "]\n");
}