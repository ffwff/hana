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
#define ERROR(code) do { vm->error = code; return; } while(0)
#define ERROR_EXPECT(code, expect) do { vm->error = code; vm->error_expected = expect; return; } while(0)
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
        // logic
        X(OP_AND), X(OP_OR),
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
        // record
        X(OP_DICT_NEW),
        X(OP_MEMBER_GET), X(OP_MEMBER_GET_NO_POP),
        X(OP_MEMBER_SET), X(OP_DICT_LOAD), X(OP_ARRAY_LOAD),
        X(OP_INDEX_GET), X(OP_INDEX_SET),
        // exceptions
        X(OP_TRY), X(OP_RAISE), X(OP_EXFRAME_RET),
        // tail calls
        X(OP_RETCALL),
        // iterators
        X(OP_FOR_IN), X(OP_SWAP),
        // modules
        X(OP_USE)
    };

#undef X

    dispatch();

    // halt
    doop(OP_HALT): {
        LOG("HALT\n");
        return;
    }

    // stack manip
    // push uint family on to the stack
#define push_int_op(optype, _type, _data) \
    doop(optype): { \
        vm->ip++; \
        const _type data = _data; \
        vm->ip += sizeof(_type); \
        LOG(sizeof(_type) == 8 ? "PUSH %ld\n" : "PUSH %d\n", data); \
\
        struct value val = { \
            .type = TYPE_INT,\
            .as.integer = data }; \
        array_push(vm->stack, val); \
        dispatch(); \
    }
    push_int_op(OP_PUSH8,  uint8_t,  vm->code.data[vm->ip+0])

    push_int_op(OP_PUSH16, uint16_t, (uint16_t)vm->code.data[vm->ip+0] << 8 |
                                     (uint16_t)vm->code.data[vm->ip+1])

    push_int_op(OP_PUSH32, uint32_t, (uint32_t)vm->code.data[vm->ip+0] << 24 |
                                     (uint32_t)vm->code.data[vm->ip+1] << 16 |
                                     (uint32_t)vm->code.data[vm->ip+2] << 8  |
                                     (uint32_t)vm->code.data[vm->ip+3])

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
        u.u[0] = vm->code.data[vm->ip+0];
        u.u[1] = vm->code.data[vm->ip+1];
        u.u[2] = vm->code.data[vm->ip+2];
        u.u[3] = vm->code.data[vm->ip+3];
        u.u[4] = vm->code.data[vm->ip+4];
        u.u[5] = vm->code.data[vm->ip+5];
        u.u[6] = vm->code.data[vm->ip+6];
        u.u[7] = vm->code.data[vm->ip+7];
        vm->ip += sizeof(u);
        LOG("PUSH_F64 %f\n", u.d);
        array_push(vm->stack, (struct value){0});
        value_float(&array_top(vm->stack), u.d);
        dispatch();
    }

    // push string on to the stack
    doop(OP_PUSHSTR): {
        vm->ip++;
        char *str = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(str)+1;
        LOG("PUSH %s\n", str);
        array_push(vm->stack, (struct value){0});
        value_str(&array_top(vm->stack), str);
        dispatch();
    }

    // push nil to the stack
    doop(OP_PUSH_NIL): {
        vm->ip++;
        LOG("PUSH NIL\n");
        array_push(vm->stack, (struct value){0});
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
        int truth = value_is_true(val);
        value_int(&val, !truth);
        array_push(vm->stack, val);
        dispatch();
    }
    // pops top of the stack, performs unary negation and pushes the result
    doop(OP_NEGATE): {
        vm->ip++;
        struct value *val = &array_top(vm->stack);
        if(val->type == TYPE_INT) {
            val->as.integer = -val->as.integer;
        } else if(val->type == TYPE_FLOAT) {
            val->as.floatp = -val->as.floatp;
        }
        dispatch();
    }

    // binary ops: perform binary operations on the 2 top values of the stack
    // arithmetic:
#define binop(optype, fn) \
    doop(optype): { \
        LOG("" #optype "\n"); \
        debug_assert(vm->stack.length >= 2); \
\
        struct value right = array_top(vm->stack); \
        array_pop(vm->stack); \
        struct value left = array_top(vm->stack); \
        array_pop(vm->stack); \
\
        array_push(vm->stack, (struct value){0}); \
        struct value *result = &array_top(vm->stack); \
        fn(result, left, right); \
        if(result->type == TYPE_INTERPRETER_ERROR) { \
            ERROR(ERROR_ ##optype); } \
        vm->ip++; \
        dispatch(); \
    }
    binop(OP_ADD, value_add)
    binop(OP_SUB, value_sub)
    binop(OP_MUL, value_mul)
    binop(OP_DIV, value_div)
    binop(OP_MOD, value_mod)

    // logic
    binop(OP_AND, value_and)
    binop(OP_OR, value_or)

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
            const struct dict *rhs = right.as.dict;
            struct value retval = {
                .as.integer = 0,
                .type = TYPE_INT
            };
            if ((rhs == vm->drec && left.type == TYPE_DICT) ||
                (value_get_prototype(vm, left) == rhs)) {
                retval.as.integer = 1;
            }
            array_push(vm->stack, retval);
        } else {
            ERROR(ERROR_EXPECTED_RECORD_OF_EXPR);
        }

        dispatch();
    }

    // variables
    // creates a new environment whenever a function is called
    // the environment is initialized with a copy of the current environment's variables
    doop(OP_ENV_NEW): {
        vm->ip++;
        const uint16_t n = vm->code.data[vm->ip+0] << 8 |
                           vm->code.data[vm->ip+1];
        vm->ip += sizeof(n);
        LOG("RESERVE %d\n", n);
        env_init(vm->localenv, n, vm);
        dispatch();
    }

    // variables
    // sets the value of current environment's slot to the top of the stack
    doop(OP_SET_LOCAL): {
        vm->ip++;
        const uint16_t key = vm->code.data[vm->ip+0] << 8 |
                             vm->code.data[vm->ip+1];
        vm->ip += sizeof(key);
        LOG("SET LOCAL %d\n", key);
        env_set(vm->localenv, key, array_top(vm->stack));
        dispatch();
    }
    // this is for recursive function
    doop(OP_SET_LOCAL_FUNCTION_DEF): {
        vm->ip++;
        const uint16_t key = vm->code.data[vm->ip+0] << 8 |
                             vm->code.data[vm->ip+1];
        vm->ip += sizeof(key);
        struct value val = array_top(vm->stack);
        env_set(vm->localenv, key, val);
        function_set_bound_var(val.as.ifn, key, val);
        dispatch();
    }
    // pushes a copy of the value of current environment's slot
    doop(OP_GET_LOCAL): {
        vm->ip++;
        const uint16_t key = vm->code.data[vm->ip+0] << 8 |
                             vm->code.data[vm->ip+1];
        vm->ip += sizeof(key);
        LOG("GET LOCAL %d\n", key);
        array_push(vm->stack, (struct value){0});
        array_top(vm->stack) = env_get(vm->localenv, key);
        dispatch();
    }
    doop(OP_GET_LOCAL_UP): {
        vm->ip++;
        const uint16_t key = vm->code.data[vm->ip+0] << 8 |
                             vm->code.data[vm->ip+1];
        vm->ip += sizeof(key);
        uint16_t relascope = vm->code.data[vm->ip+0] << 8 |
                             vm->code.data[vm->ip+1];
        vm->ip += sizeof(relascope);
        LOG("GET LOCAL UP %d %d\n", key, relascope);

        array_push(vm->stack, (struct value){0});
        array_top(vm->stack) = env_get_up(vm->localenv, relascope, key);
        dispatch();
    }

    // sets the value of the global variable to the top of the stack
    doop(OP_SET_GLOBAL): {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("SET GLOBAL %s %p\n", key, vm->globalenv);
        hmap_set(vm->globalenv, key, array_top(vm->stack));
        dispatch();
    }
    // pushes a copy of the value of the global variable
    doop(OP_GET_GLOBAL): {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("GET GLOBAL %s\n", key);
        const struct value *val = hmap_get(vm->globalenv, key);
        if(val == NULL) {
            ERROR(ERROR_UNDEFINED_GLOBAL_VAR);
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
        const uint16_t nargs = (uint16_t)vm->code.data[vm->ip + 0] << 8 |
                               (uint16_t)vm->code.data[vm->ip + 1];
        vm->ip += sizeof(nargs);
        const uint16_t pos = (uint16_t)vm->code.data[vm->ip + 0] << 8 |
                             (uint16_t)vm->code.data[vm->ip + 1];
        LOG("DEF_FUNCTION_PUSH %d %d\n", pos, nargs);
        array_push(vm->stack, (struct value){0});
        value_function(&array_top(vm->stack), vm->ip + sizeof(pos), nargs, vm->localenv);
        vm->ip += pos;
        dispatch();
    }

    // flow control
    doop(OP_JMP): { // jmp [32-bit position]
        vm->ip++;
        const int16_t pos = (uint16_t)vm->code.data[vm->ip+0] << 8 |
                            (uint16_t)vm->code.data[vm->ip+1];
        LOG("JMP %d\n", pos);
        vm->ip += pos;
        dispatch();
    }
    doop(OP_JMP_LONG): { // jmp [32-bit position]
        vm->ip++;
        const uint32_t pos = (uint32_t)vm->code.data[vm->ip+0] << 24 |
                             (uint32_t)vm->code.data[vm->ip+1] << 16 |
                             (uint32_t)vm->code.data[vm->ip+2] << 8  |
                             (uint32_t)vm->code.data[vm->ip+3];
        LOG("JMP LONG %d\n", pos);
        vm->ip = pos;
        dispatch();
    }
    doop(OP_JCOND): { // jmp if not true [32-bit position]
        vm->ip++;
        const int16_t pos = (uint16_t)vm->code.data[vm->ip+0] << 8 |
                            (uint16_t)vm->code.data[vm->ip+1];
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        LOG("JCOND %d\n", pos);
        if(value_is_true(val)) vm->ip += pos;
        else vm->ip += sizeof(pos);
        dispatch();
    }
    doop(OP_JNCOND): { // jump if true [32-bit position]
        vm->ip++;
        const int16_t pos = (uint16_t)vm->code.data[vm->ip+0] << 8 |
                            (uint16_t)vm->code.data[vm->ip+1];
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        LOG("JNCOND %d\n", pos);
        if(!value_is_true(val)) vm->ip += pos;
        else vm->ip += sizeof(pos);
        dispatch();
    }
    // pops a function/record constructor on top of the stack,
    // sets up necessary environment and calls it.
#define CALL_NATIVE(expr)        \
    do {                         \
        vm->native_call_depth++; \
        expr(vm, nargs);         \
        vm->native_call_depth--; \
        if(vm->exframe_fallthrough != NULL) \
            vm->exframe_fallthrough = NULL; \
        else if(vm->error) return;    \
    } while(0)
#define JMP_INTERPRETED_FN(END_IF_NATIVE)                                    \
    do {                                                                     \
        if (val.type == TYPE_DICT) {                                         \
            array_pop(vm->stack);                                            \
            const struct value *ctor = dict_get(val.as.dict, "constructor"); \
            if (ctor == NULL) {                                              \
                ERROR(ERROR_RECORD_NO_CONSTRUCTOR);                          \
            }                                                                \
            if (ctor->type == TYPE_NATIVE_FN) {                              \
                LOG("NATIVE CONSTRUCTOR %d\n", nargs);                       \
                CALL_NATIVE(ctor->as.fn);                                    \
                do { END_IF_NATIVE } while(0);                               \
            } else if (ctor->type != TYPE_FN) {                              \
                ERROR(ERROR_CONSTRUCTOR_NOT_FUNCTION);                       \
            }                                                                \
            ifn = ctor->as.ifn;                                              \
            if (nargs + 1 != ifn->nargs) {                                   \
                ERROR_EXPECT(ERROR_MISMATCH_ARGUMENTS, ifn->nargs);          \
            }                                                                \
            struct value new_val;                                            \
            value_dict(&new_val);                                            \
            dict_set(new_val.as.dict, "prototype", val);                     \
            array_push(vm->stack, new_val);                                  \
        } else {                                                             \
            ifn = val.as.ifn;                                                \
            array_pop(vm->stack);                                            \
            if (nargs != ifn->nargs) {                                       \
                ERROR_EXPECT(ERROR_MISMATCH_ARGUMENTS, ifn->nargs);          \
            }                                                                \
        }                                                                    \
    } while (0)
    doop(OP_CALL): {
        // argument: [arg2][arg1]
        vm->ip++;
        struct value val = array_top(vm->stack);
        const uint16_t nargs = vm->code.data[vm->ip+0] << 8 |
                               vm->code.data[vm->ip+1];
        vm->ip += sizeof(nargs);
        debug_assert(vm->stack.length >= nargs);
        LOG("call %d\n", nargs);
        switch(val.type) {
        case TYPE_NATIVE_FN: {
            array_pop(vm->stack);
            CALL_NATIVE(val.as.fn);
            break; }
        case TYPE_FN:
        case TYPE_DICT: {
            struct function *ifn;
            JMP_INTERPRETED_FN(
                if (vm->exframe_fallthrough != NULL) {
                    if (exframe_native_stack_depth(vm->exframe_fallthrough) == vm->native_call_depth) {
                        assert(0);
                    } else {
                        // just unwind
                        return;
                    }
                }
                dispatch();
            );

            // caller
            vm_enter_env(vm, ifn);
            break; }
        default: {
            ERROR(ERROR_EXPECTED_CALLABLE); }
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
        dispatch();
    }

    // dictionaries
    doop(OP_DICT_NEW): {
        vm->ip++;
        LOG("DICT_NEW\n");
        array_push(vm->stack, (struct value){0});
        value_dict(&array_top(vm->stack));
        dispatch();
    }
    doop(OP_MEMBER_GET):
    doop(OP_MEMBER_GET_NO_POP): {
        const enum vm_opcode op = vm->code.data[vm->ip];
        vm->ip++;
        const char *key = (const char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG(op == OP_MEMBER_GET ? "MEMBER_GET %s\n" : "MEMBER_GET_NO_POP %s\n", key);

        struct value val = array_top(vm->stack);
        struct dict *dict = NULL;
        if(val.type != TYPE_DICT) {
            if((dict = value_get_prototype(vm, val)) == NULL) {
                ERROR(ERROR_CANNOT_ACCESS_NON_RECORD);
            }
            if(strcmp(key, "prototype") == 0) {
                struct value *val = &array_top(vm->stack);
                val->type = TYPE_DICT;
                val->as.dict = dict;
                dispatch();
            }
        } else {
            if(val.type != TYPE_DICT) {
                ERROR(ERROR_CANNOT_ACCESS_NON_RECORD);
            }
            dict = val.as.dict;
            if(op == OP_MEMBER_GET) array_pop(vm->stack);
        }

        const struct value *result = dict_get(dict, key);
        if(result != NULL) {
            array_push(vm->stack, *result);
        } else {
            array_push(vm->stack, (struct value){0});
        }

        dispatch();
    }
    doop(OP_MEMBER_SET): {
        // stack: [value][dict]
        vm->ip++;
        char *key = (char *)(vm->code.data+vm->ip); // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("MEMBER_SET %s\n", key);
        struct value dval = array_top(vm->stack);
        if(dval.type != TYPE_DICT) {
            ERROR(ERROR_CANNOT_ACCESS_NON_RECORD);
        }
        array_pop(vm->stack);

        struct value val = array_top(vm->stack);
        dict_set(dval.as.dict, key, val);
        dispatch();
    }
    doop(OP_DICT_LOAD): {
        // stack: [nil][value][key]
        LOG("dict load\n");
        vm->ip++;
        struct value dval;
        value_dict(&dval);

        struct value key = {0};
        while((key = array_top(vm->stack)).type != TYPE_NIL) {
            debug_assert(key.type == TYPE_STR);
            // key
            array_pop(vm->stack);
            // val
            struct value val = array_top(vm->stack);
            array_pop(vm->stack);
            dict_set_str(dval.as.dict, key.as.str, val);
        }

        array_pop(vm->stack);  // pop nil
        array_push(vm->stack, dval);
        dispatch();
    }
    // array
    doop(OP_INDEX_GET): {
        vm->ip++;
        LOG("INDEX_GET\n");

        const struct value index = array_top(vm->stack);
        array_pop(vm->stack);

        const struct value dval = array_top(vm->stack);
        array_pop(vm->stack);

        if(dval.type == TYPE_ARRAY) {
            if(index.type != TYPE_INT) {
                ERROR(ERROR_KEY_NON_INT);
            }
            const int64_t i = (int64_t)index.as.integer;
            if(!(i >= 0 && i < (int64_t)dval.as.array->length)) {
                ERROR_EXPECT(ERROR_UNBOUNDED_ACCESS, dval.as.array->length);
            }
            array_push(vm->stack, dval.as.array->data[i]);
        } else if(dval.type == TYPE_STR) {
            if(index.type != TYPE_INT) {
                ERROR(ERROR_KEY_NON_INT);
            }
            const int64_t i = index.as.integer;
            struct value val;
            val.type = TYPE_STR;
            val.as.str = string_at(dval.as.str, i);
            // TODO bounds check
            array_push(vm->stack, val);
        } else if(dval.type == TYPE_DICT) {
            if(index.type != TYPE_STR) {
                ERROR(ERROR_RECORD_KEY_NON_STRING);
            }
            const struct value *val = dict_get_str(dval.as.dict, index.as.str);
            if(val != NULL) {
                array_push(vm->stack, *val);
            } else {
                array_push(vm->stack, (struct value){0});
            }
        } else {
            ERROR(ERROR_CANNOT_ACCESS_NON_RECORD);
        }
        dispatch();
    }
    doop(OP_INDEX_SET): {
        vm->ip++;
        LOG("INDEX_SET\n");

        struct value index = array_top(vm->stack);
        array_pop(vm->stack);

        struct value dval = array_top(vm->stack);
        array_pop(vm->stack);

        struct value val = array_top(vm->stack);

        if(dval.type == TYPE_ARRAY) {
            if(index.type != TYPE_INT) {
                ERROR(ERROR_KEY_NON_INT);
            }
            const int64_t i = index.as.integer;
            if (!(i >= 0 && i < (int64_t)dval.as.array->length)) {
                ERROR_EXPECT(ERROR_UNBOUNDED_ACCESS, dval.as.array->length);
            }
            dval.as.array->data[i] = val;
        } else if(dval.type == TYPE_DICT) {
            if(index.type != TYPE_STR) {
                ERROR(ERROR_RECORD_KEY_NON_STRING);
            }
            dict_set_str(dval.as.dict, index.as.str, val);
        } else {
            ERROR(ERROR_EXPECTED_RECORD_ARRAY);
        }
        dispatch();
    }
    doop(OP_ARRAY_LOAD): {
        vm->ip++;

        struct value val = array_top(vm->stack);
        int64_t length = val.as.integer;
        array_pop(vm->stack);
        debug_assert(val.type == TYPE_INT);
        LOG("ARRAY_LOAD %ld\n", length);

        struct value aval;
        if(length == 0) {
            value_array(&aval);
        } else {
            value_array_n(&aval, length);
            aval.as.array->length = length;
            while(length--) {
                aval.as.array->data[length] = array_top(vm->stack);
                array_pop(vm->stack);
            }
        }
        array_push(vm->stack, aval);
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
                ERROR(ERROR_CASE_EXPECTS_DICT);
            }
            array_pop(vm->stack);
            // val
            struct value fn = array_top(vm->stack);
            debug_assert(fn.type == TYPE_FN);
            array_pop(vm->stack);
            exframe_set_handler(frame, error.as.dict, fn.as.ifn);
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
        if (vm->exframe_fallthrough != NULL) {
            LOG("falling through pls wait (%ld)\n", vm->native_call_depth);
            return;
        }
        dispatch();
    }
    doop(OP_EXFRAME_RET): {
        vm->ip++;
        const uint16_t pos = (uint16_t)vm->code.data[vm->ip + 0] << 8 |
                             (uint16_t)vm->code.data[vm->ip + 1];
        LOG("EXFRAME_RET %d\n", pos);
        vm_leave_exframe(vm);
        vm->ip += pos;
        dispatch();
    }

    // tail calls
    doop(OP_RETCALL): {
        vm->ip++;
        struct value val = array_top(vm->stack);
        const uint16_t nargs = (uint16_t)vm->code.data[vm->ip+0] << 8 |
                               (uint16_t)vm->code.data[vm->ip+1];
        vm->ip += sizeof(nargs);
        debug_assert(vm->stack.length >= nargs);
        LOG("retcall %d\n", nargs);
        switch(val.type) {
        case TYPE_NATIVE_FN: {
            array_pop(vm->stack);
            CALL_NATIVE(val.as.fn);
            if (vm_leave_env(vm)) {
                LOG("return from vm_call\n");
                return;
            }
            break; }
        case TYPE_FN:
        case TYPE_DICT: {
            struct function *ifn;
            JMP_INTERPRETED_FN(
                if (vm_leave_env(vm)) {
                    LOG("return from vm_call\n");
                    return;
                } else {
                    dispatch();
                }
            );

            // caller
            vm_enter_env_tail(vm, ifn);
            break; }
        default: {
            ERROR(ERROR_EXPECTED_CALLABLE); }
        }
        dispatch();
    }

    // operators
    doop(OP_FOR_IN): {
        vm->ip++;
        const uint16_t pos = (uint16_t)vm->code.data[vm->ip + 0] << 8 |
                             (uint16_t)vm->code.data[vm->ip + 1];

        LOG("FOR_IN %d\n", pos);
        struct value *top = &array_top(vm->stack);
        if (top->type == TYPE_INTERPRETER_ITERATOR) {
            const array_obj *array = vm->stack.data[vm->stack.length-2].as.array;
            const size_t idx = (size_t)top->as.integer;
            if (idx == array->length) { // end of it
                array_pop(vm->stack); // iterator
                array_pop(vm->stack); // array
                vm->ip += pos;
                dispatch();
            } else {
                LOG("nop!\n");
                top->as.integer++;
                array_push(vm->stack, array->data[idx]);
            }
        } else if (top->type == TYPE_ARRAY) {
            struct value val = {
                .as.integer = 1,
                .type = TYPE_INTERPRETER_ITERATOR
            };
            array_push(vm->stack, val);
            array_push(vm->stack, top->as.array->data[0]);
        } else {
            ERROR(ERROR_EXPECTED_ITERABLE);
        }
        vm->ip += sizeof(pos);
        dispatch();
    }

    doop(OP_SWAP): {
        vm->ip++;
        debug_assert(vm->stack.length >= 2);
        const struct value lower = vm->stack.data[vm->stack.length-2];
        const struct value higher = vm->stack.data[vm->stack.length-1];
        vm->stack.data[vm->stack.length - 1] = lower;
        vm->stack.data[vm->stack.length - 2] = higher;
        dispatch();
    }

    // modules
    doop(OP_USE): {
        vm->ip++;
        char *str = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(str)+1;
        vm_load_module(vm, str);
        dispatch();
    }
}

struct value vm_call(struct vm *vm, const struct value fn, const a_arguments *args) {
    //debug_assert(fn->type ==  || fn->type == TYPE_DICT);

    static struct value errorval;
    errorval.type = TYPE_INTERPRETER_ERROR;

    struct function *ifn = NULL;

    if(fn.type == TYPE_DICT) {
        const struct value *ctor = dict_get(fn.as.dict, "constructor");
        if(ctor == NULL) {
            FATAL("expected record to have constructor\n");
            return errorval;
        }
        if(ctor->type == TYPE_NATIVE_FN) {
            for(int64_t i = args->length-1; i >= 0; i--) {
                array_push(vm->stack, args->data[i]);
            }
            ctor->as.fn(vm, args->length);
            if(vm->error) return errorval;
            const struct value val = array_top(vm->stack);
            array_pop(vm->stack);
            return val;
        } else if(ctor->type != TYPE_FN) {
            FATAL("constructor must be a function!\n");
            return errorval;
        } else {
            ifn = ctor->as.ifn;
        }
    } else if (fn.type == TYPE_FN) {
        ifn = fn.as.ifn;
    } else {
        FATAL("expected caller to be function");
        return errorval;
    }

    if((uint32_t)args->length != ifn->nargs) {
        FATAL("function requires %d arguments, got %ld\n", ifn->nargs, args->length);
        return errorval;
    }

    const uint32_t last = vm->ip;
    // setup env
    struct env *oldenv = vm->localenv;
    vm->ip = (uint32_t)-1;
    struct env *curenv = vm_enter_env(vm, ifn);
    // setup stack/ip
    for(int64_t i = args->length-1; i >= 0; i--) {
        array_push(vm->stack, args->data[i]);
    }
    // call it
    vm_execute(vm);
    if(vm->error || vm->exframe_fallthrough != NULL) // exception
        return errorval;
    if(vm->localenv != curenv) { // exception occurred outside of function's scope
        // NOTE: curenv already free'd from unwinding
        return errorval;
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

// push bits
void vm_code_push8(struct vm *vm, uint8_t n) {
    array_push(vm->code, n);
}

void vm_code_pushstr(struct vm *vm, const char *s) {
    for(int i = 0; s[i]; i++)
        array_push(vm->code, s[i]);
    array_push(vm->code, 0);
}

void vm_code_pushf32(struct vm *vm, float f) {
    union {
        float f;
        uint8_t u[4];
    } u;
    u.f = f;
    array_push(vm->code, u.u[0]);
    array_push(vm->code, u.u[1]);
    array_push(vm->code, u.u[2]);
    array_push(vm->code, u.u[3]);
}
void vm_code_pushf64(struct vm *vm, double d) {
    union {
        double d;
        uint8_t u[8];
    } u;
    u.d = d;
    array_push(vm->code, u.u[0]);
    array_push(vm->code, u.u[1]);
    array_push(vm->code, u.u[2]);
    array_push(vm->code, u.u[3]);
    array_push(vm->code, u.u[4]);
    array_push(vm->code, u.u[5]);
    array_push(vm->code, u.u[6]);
    array_push(vm->code, u.u[7]);
}

void vm_code_fill(struct vm *vm, const uint32_t pos, const uint32_t label) {
    vm->code.data[pos + 0] = (label >> 24) & 0xff;
    vm->code.data[pos + 1] = (label >> 16) & 0xff;
    vm->code.data[pos + 2] = (label >> 8) & 0xff;
    vm->code.data[pos + 3] = (label >> 0) & 0xff;
}
void vm_code_fill16(struct vm *vm, const uint32_t pos, const uint16_t label) {
    vm->code.data[pos + 0] = (label >> 8) & 0xff;
    vm->code.data[pos + 1] = (label >> 0) & 0xff;
}