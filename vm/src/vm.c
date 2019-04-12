#include <stdio.h>
#include <assert.h>
#include <string.h>
#include "vm.h"

#ifdef NOLOG
#define LOG(...)
#else
#define LOG(fmt, ...) do { printf(fmt __VA_OPT__(,) __VA_ARGS__); } while(0)
#endif
#define FATAL(fmt, ...) printf(fmt __VA_OPT__(,) __VA_ARGS__)

// notes: architecture is big endian!

void vm_init(struct vm *vm) {
    vm->env = malloc(sizeof(struct env));
    env_init(vm->env, NULL);
    vm->code = (a_uint8)array_init(uint8_t);
    vm->stack = (a_value)array_init(struct value);
    vm->ip = 0;
}

void vm_free(struct vm *vm) {
    env_free(vm->env);
    free(vm->env);

    array_free(vm->code);
    for(size_t i = 0; i < vm->stack.length; i++)
        value_free(&vm->stack.data[i]);
    array_free(vm->stack);
}

int vm_step(struct vm *vm) {
    enum vm_opcode op = vm->code.data[vm->ip];
    if(op == OP_HALT) {
        LOG("HALT\n");
#ifndef NOLOG
        vm_print_stack(vm);
#endif
        return 0;
    }

    // stack manip
    // push int
#define push_int_op(optype, type, _data) \
    else if (op == optype) { \
        vm->ip++; \
        const type data = _data; \
        vm->ip += sizeof(type); \
        LOG(sizeof(type) == 8 ? "PUSH %ld\n" : "PUSH %d\n", data); \
\
        array_push(vm->stack, (struct value){}); \
        value_int(&array_top(vm->stack), data); \
    }
    push_int_op(OP_PUSH8,  uint8_t,  vm->code.data[vm->ip+0])

    push_int_op(OP_PUSH16, uint16_t, vm->code.data[vm->ip+0] << 4 |
                                     vm->code.data[vm->ip+1])

    push_int_op(OP_PUSH32, uint32_t, vm->code.data[vm->ip+0] << 12 |
                                     vm->code.data[vm->ip+1] << 8  |
                                     vm->code.data[vm->ip+2] << 4  |
                                     vm->code.data[vm->ip+3])

    push_int_op(OP_PUSH64, uint64_t, vm->code.data[vm->ip+0] << 28 |
                                     vm->code.data[vm->ip+1] << 24 |
                                     vm->code.data[vm->ip+2] << 20 |
                                     vm->code.data[vm->ip+3] << 16 |
                                     vm->code.data[vm->ip+4] << 12 |
                                     vm->code.data[vm->ip+5] << 8  |
                                     vm->code.data[vm->ip+6] << 4  |
                                     vm->code.data[vm->ip+7])
    // pushstr
    else if(op == OP_PUSHSTR) {
        vm->ip++;
        char *str = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(str)+1;
        LOG("PUSH %s\n", str);
        array_push(vm->stack, (struct value){});
        value_str(&array_top(vm->stack), str);
    }

    // pop
    else if (op == OP_POP) {
        LOG("POP\n");
        assert(vm->stack.length > 0);
        vm->ip++;
        value_free(&array_top(vm->stack));
        array_pop(vm->stack);
    }

    // arith
#define binop(optype, fn) \
    else if (op == optype) { \
        LOG("" #optype "\n"); \
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

    // variables
    else if(op == OP_SET) {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("SET %s\n", key);
        env_set(vm->env, key, &array_top(vm->stack));
    }
    else if(op == OP_SET_LOCAL) {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("SET_LOCAL %s\n", key);
        env_set_local(vm->env, key, &array_top(vm->stack));
    }
    else if(op == OP_GET) {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("GET %s\n", key);
        array_push(vm->stack, (struct value){});
        struct value *val = env_get(vm->env, key);
        if(val == NULL) {
            FATAL("no key named %s!\n", key);
            return 0;
        } else
            value_copy(&array_top(vm->stack), val);
    }
    else if(op == OP_INC || op == OP_DEC) {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        if(op == OP_INC) LOG("INC %s\n", key);
        else if(op == OP_DEC) LOG("DEC %s\n", key);
        struct value *val = env_get(vm->env, key);
        if(val->type == TYPE_INT) {
            if(op == OP_INC) val->as.integer++;
            else if(op == OP_DEC) val->as.integer--;
        } else if(val->type == TYPE_FLOAT) {
            if(op == OP_INC) val->as.floatp++;
            else if(op == OP_DEC) val->as.floatp--;
        } else {
            FATAL("must be int or float!\n");
            assert(0);
        }
    }
    else if(op == OP_DEF_FUNCTION) {
        // [opcode][key][end address]
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        const uint64_t pos = vm->code.data[vm->ip+0] << 28 |
                             vm->code.data[vm->ip+1] << 24 |
                             vm->code.data[vm->ip+2] << 20 |
                             vm->code.data[vm->ip+3] << 16 |
                             vm->code.data[vm->ip+4] << 12 |
                             vm->code.data[vm->ip+5] << 8  |
                             vm->code.data[vm->ip+6] << 4  |
                             vm->code.data[vm->ip+7];
        LOG("DEF_FUNCTION %s %ld\n", key, pos);
        vm->ip += 8;
        struct value val;
        LOG("%ld\n", vm->ip);
        value_function(&val, vm->ip);
        env_set(vm->env, key, &val);
        //map_set(&vm->env, key, &val);
        vm->ip = pos;
    }

    // flow control
    else if(op == OP_JMP) { // jmp [64-bit position]
        vm->ip++;
        const uint64_t pos = vm->code.data[vm->ip+0] << 28 |
                             vm->code.data[vm->ip+1] << 24 |
                             vm->code.data[vm->ip+2] << 20 |
                             vm->code.data[vm->ip+3] << 16 |
                             vm->code.data[vm->ip+4] << 12 |
                             vm->code.data[vm->ip+5] << 8  |
                             vm->code.data[vm->ip+6] << 4  |
                             vm->code.data[vm->ip+7];
        LOG("JMP %ld\n", pos);
        vm->ip = pos;
    }
    else if(op == OP_JCOND || op == OP_JNCOND) { // jcond [64-bit position]
        vm->ip++;
        const uint64_t pos = vm->code.data[vm->ip+0] << 28 |
                             vm->code.data[vm->ip+1] << 24 |
                             vm->code.data[vm->ip+2] << 20 |
                             vm->code.data[vm->ip+3] << 16 |
                             vm->code.data[vm->ip+4] << 12 |
                             vm->code.data[vm->ip+5] << 8  |
                             vm->code.data[vm->ip+6] << 4  |
                             vm->code.data[vm->ip+7];
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        if(op == OP_JCOND) {
            LOG("JCOND %ld\n", pos);
            if(value_is_true(&val)) vm->ip = pos;
            else vm->ip += 8;
        } else {
            LOG("JNCOND %ld\n", pos);
            if(!value_is_true(&val)) vm->ip = pos;
            else vm->ip += 8;
        }
    }
    else if(op == OP_CALL) {
        vm->ip++;
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        int nargs = vm->code.data[vm->ip++];
        assert(vm->stack.length >= nargs);
        LOG("call %d\n", nargs);
        if(val.type == TYPE_NATIVE_FN) {
            val.as.fn(vm, nargs);
        } else if(val.type == TYPE_FN) {
            struct value args[nargs];
            for(int i = 0; i < nargs; i++) {
                struct value val = array_top(vm->stack);
                array_pop(vm->stack);
                args[i] = val;
            }
            // caller
            struct value caller;
            value_function(&caller, vm->ip);
            array_push(vm->stack, caller);
            // arguments
            for(int i = 0; i < nargs; i++) {
                array_push(vm->stack, args[i]);
            }
            struct value nargs_v;
            value_int(&nargs_v, nargs);
            array_push(vm->stack, nargs_v);
            vm->ip = val.as.fn_ip;
        } else {
            printf("is not a function\n");
            return 0;
        }
    }
    else if(op == OP_RET) {
        struct value retval = array_top(vm->stack);
        array_pop(vm->stack);

        struct value caller = array_top(vm->stack);
        array_pop(vm->stack);
        assert(caller.type == TYPE_FN);

        LOG("RET\n");
        vm->ip = caller.as.fn_ip;
        array_push(vm->stack, retval);

        assert(vm->env->parent != NULL);
        struct env *parent = vm->env->parent;
        env_free(vm->env);
        free(vm->env);
        vm->env = parent;
    }

    // scoped
    else if (op == OP_ENV_INHERIT) {
        vm->ip++;
        LOG("ENV_INHERIT\n");
        struct env *parent = vm->env;
        vm->env = malloc(sizeof(struct env));
        env_init(vm->env, parent);
    }
    else if (op == OP_ENV_POP) {
        vm->ip++;
        LOG("ENV_POP\n");
        assert(vm->env->parent != NULL);
        struct env *parent = vm->env->parent;
        env_free(vm->env);
        free(vm->env);
        vm->env = parent;
    }

    // end
    else {
        FATAL("undefined opcode: %d\n", op);
        assert(0);
    }

#ifndef NOLOG
    vm_print_stack(vm);
#endif
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
    array_push(vm->code, (n >> 4) & 0xff);
    array_push(vm->code, (n >> 0) & 0xff);
}

void vm_code_push32(struct vm *vm, uint32_t n) {
    array_push(vm->code, (n >> 12) & 0xff);
    array_push(vm->code, (n >> 8)  & 0xff);
    array_push(vm->code, (n >> 4)  & 0xff);
    array_push(vm->code, (n >> 0)  & 0xff);
}

void vm_code_push64(struct vm *vm, uint64_t n) {
    array_push(vm->code, (n >> 28) & 0xff);
    array_push(vm->code, (n >> 24) & 0xff);
    array_push(vm->code, (n >> 20) & 0xff);
    array_push(vm->code, (n >> 16) & 0xff);
    array_push(vm->code, (n >> 12) & 0xff);
    array_push(vm->code, (n >> 8)  & 0xff);
    array_push(vm->code, (n >> 4)  & 0xff);
    array_push(vm->code, (n >> 0)  & 0xff);
}

void vm_code_pushstr(struct vm *vm, const char *s) {
    for(int i = 0; s[i]; i++)
        array_push(vm->code, s[i]);
    array_push(vm->code, 0);
}
