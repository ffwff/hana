#include <stdio.h>
//#include <assert.h>
#include <string.h>
#include "vm.h"
#include "string_.h"
#include "dict.h"
#include "array_obj.h"

#define assert(...)
#ifdef NOLOG
#define LOG(...)
#else
#define LOG(fmt, ...) do { printf(fmt __VA_OPT__(,) __VA_ARGS__); } while(0)
#endif
#define FATAL printf

// notes: architecture is big endian!

void vm_init(struct vm *vm) {
    hmap_init(&vm->globalenv);
    vm->localenv = NULL;
    vm->code = (a_uint8)array_init(uint8_t);
    vm->stack = (a_value){
        .data = calloc(8, sizeof(struct value)),
        .length = 0,
        .capacity = 8
    };
    vm->ip = 0;
    vm->dstr = vm->dint = vm->dfloat = vm->darray = 0;
}

void vm_free(struct vm *vm) {
    hmap_free(&vm->globalenv);
    // TODO free localenv
    while(vm->localenv != NULL) {
        struct env *localenv = vm->localenv->parent;
        env_free(vm->localenv);
        free(vm->localenv);
        vm->localenv = localenv;
    }
    array_free(vm->code);
    for(size_t i = 0; i < vm->stack.length; i++)
        value_free(&vm->stack.data[i]);
    array_free(vm->stack);
}

void vm_execute(struct vm *vm) {
#define doop(op) do_ ## op
#define X(op) [op] = && doop(op)
#ifdef NOLOG
#define dispatch() do { \
        goto *dispatch_table[vm->code.data[vm->ip]]; \
    } while(0)
#else
#define dispatch() do { \
        vm_print_stack(vm); \
        goto *dispatch_table[vm->code.data[vm->ip]]; \
    } while(0)
#endif

    static const void *dispatch_table[] = {
        X(OP_HALT),
        // stack manip
        X(OP_PUSH8), X(OP_PUSH16), X(OP_PUSH32), X(OP_PUSH64),
        X(OP_PUSH_NIL), X(OP_PUSHSTR), X(OP_PUSHF32), X(OP_PUSHF64),
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
        // variables
        X(OP_ENV_NEW),
        X(OP_SET_LOCAL), X(OP_SET_GLOBAL), X(OP_GET_LOCAL), X(OP_GET_GLOBAL),
        X(OP_DEF_FUNCTION), X(OP_DEF_FUNCTION_PUSH),
        // flow control
        X(OP_JMP), X(OP_JCOND), X(OP_JNCOND), X(OP_CALL), X(OP_RET),
        // dictionary
        X(OP_DICT_NEW), X(OP_MEMBER_GET), X(OP_MEMBER_GET_NO_POP),
        X(OP_MEMBER_SET), X(OP_DICT_LOAD), X(OP_ARRAY_LOAD),
        X(OP_INDEX_GET), X(OP_INDEX_SET),
    };

#undef X

    dispatch();
    while(1) {
    // halt
    doop(OP_HALT): {
        LOG("HALT\n");
        return;
    }

    // stack manip
    // push int
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

    // push float
    doop(OP_PUSHF32): {
        vm->ip++;
        union {
            float f;
            uint8_t u[4];
        } u;
        u.u[0] = vm->code.data[vm->ip+0];
        u.u[1] = vm->code.data[vm->ip+1];
        u.u[2] = vm->code.data[vm->ip+2];
        u.u[3] = vm->code.data[vm->ip+3];
        vm->ip += sizeof(u);
        LOG("PUSH_F32 %f\n", u.f);
        array_push(vm->stack, (struct value){});
        value_float(&array_top(vm->stack), u.f);
        dispatch();
    }
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
        array_push(vm->stack, (struct value){});
        value_float(&array_top(vm->stack), u.d);
        dispatch();
    }

    // pushstr
    doop(OP_PUSHSTR): {
        vm->ip++;
        char *str = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(str)+1;
        LOG("PUSH %s\n", str);
        array_push(vm->stack, (struct value){});
        value_str(&array_top(vm->stack), str);
        dispatch();
    }

    // nil
    doop(OP_PUSH_NIL): {
        vm->ip++;
        LOG("PUSH NIL\n");
        array_push(vm->stack, (struct value){});
        dispatch();
    }

    // pop
    doop(OP_POP): {
        LOG("POP\n");
        assert(vm->stack.length > 0);
        vm->ip++;
        value_free(&array_top(vm->stack));
        array_pop(vm->stack);
        dispatch();
    }

    // unary
    doop(OP_NOT): {
        vm->ip++;
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        int truth = value_is_true(&val);
        value_free(&val);
        value_int(&val, !truth);
        array_push(vm->stack, val);
        dispatch();
    }
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

    // arith
#define binop(optype, fn) \
    doop(optype): { \
        LOG("" #optype "\n"); \
        assert(vm->stack.length >= 2); \
        vm->ip++; \
\
        struct value right = array_top(vm->stack); \
        array_pop(vm->stack); \
        struct value left = array_top(vm->stack); \
        array_pop(vm->stack); \
\
        array_push(vm->stack, (struct value){}); \
        struct value *result = &array_top(vm->stack); \
        fn(result, &left, &right); \
        value_free(&left); value_free(&right); \
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

    // scope
    doop(OP_ENV_NEW): {
        vm->ip++;
        const uint32_t n =
            vm->code.data[vm->ip+0] << 12 |
            vm->code.data[vm->ip+1] << 8  |
            vm->code.data[vm->ip+2] << 4  |
            vm->code.data[vm->ip+3];
        vm->ip += sizeof(n);
        LOG("RESERVE %d\n", n);
        env_init(vm->localenv, n);
        dispatch();
    }

    // variables
    doop(OP_SET_LOCAL): {
        vm->ip++;
        const uint32_t key =
                            vm->code.data[vm->ip+0] << 12 |
                            vm->code.data[vm->ip+1] << 8  |
                            vm->code.data[vm->ip+2] << 4  |
                            vm->code.data[vm->ip+3];
        vm->ip += sizeof(key);
        LOG("SET LOCAL %d\n", key);
        env_set(vm->localenv, key, &array_top(vm->stack));
        dispatch();
    }
    doop(OP_SET_GLOBAL): {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        LOG("SET GLOBAL %s\n", key);
        hmap_set(&vm->globalenv, key, &array_top(vm->stack));
        dispatch();
    }
    doop(OP_GET_LOCAL): {
        vm->ip++;
        const uint32_t key =
                            vm->code.data[vm->ip+0] << 12 |
                            vm->code.data[vm->ip+1] << 8  |
                            vm->code.data[vm->ip+2] << 4  |
                            vm->code.data[vm->ip+3];
        vm->ip += sizeof(key);
        LOG("GET LOCAL %d\n", key);
        array_push(vm->stack, (struct value){});
        value_copy(&array_top(vm->stack), env_get(vm->localenv, key));
        dispatch();
    }
    doop(OP_GET_GLOBAL): {
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        const uint32_t hash =
                            vm->code.data[vm->ip+0] << 12 |
                            vm->code.data[vm->ip+1] << 8  |
                            vm->code.data[vm->ip+2] << 4  |
                            vm->code.data[vm->ip+3];
        vm->ip+=sizeof(hash);
        LOG("GET GLOBAL %s\n", key);
        array_push(vm->stack, (struct value){});
        struct value *val = hmap_get(&vm->globalenv, key);
        if(val == NULL) {
            FATAL("no key named %s!\n", key);
            return;
        } else
            value_copy(&array_top(vm->stack), val);
        dispatch();
    }
    doop(OP_DEF_FUNCTION): {
        // [opcode][key][end address]
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        const uint32_t pos = vm->code.data[vm->ip+0] << 12 |
        vm->code.data[vm->ip+1] << 8  |
        vm->code.data[vm->ip+2] << 4  |
        vm->code.data[vm->ip+3];
        vm->ip += 4;
        uint32_t nargs = vm->code.data[vm->ip++];
        LOG("DEF_FUNCTION %s %d %d\n", key, pos, nargs);
        struct value val;
        value_function(&val, vm->ip, nargs);
        vm->ip = pos;
        hmap_set(&vm->globalenv, key, &val);
        dispatch();
    }

    doop(OP_DEF_FUNCTION_PUSH): {
        // [opcode][key][end address]
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        const uint32_t pos = vm->code.data[vm->ip+0] << 12 |
                             vm->code.data[vm->ip+1] << 8  |
                             vm->code.data[vm->ip+2] << 4  |
                             vm->code.data[vm->ip+3];
        vm->ip += 4;
        uint32_t nargs = vm->code.data[vm->ip++];
        LOG("DEF_FUNCTION_PUSH %s %d %d\n", key, pos, nargs);
        struct value val;
        value_function(&val, vm->ip, nargs);
        vm->ip = pos;
        array_push(vm->stack, val);
        dispatch();
    }

    // flow control
    doop(OP_JMP): { // jmp [64-bit position]
        vm->ip++;
        const uint64_t pos = vm->code.data[vm->ip+0] << 12 |
                             vm->code.data[vm->ip+1] << 8  |
                             vm->code.data[vm->ip+2] << 4  |
                             vm->code.data[vm->ip+3];
        LOG("JMP %ld\n", pos);
        vm->ip = pos;
        dispatch();
    }
    doop(OP_JCOND): { // jcond [64-bit position]
        vm->ip++;
        const uint32_t pos = vm->code.data[vm->ip+0] << 12 |
                            vm->code.data[vm->ip+1] << 8  |
                            vm->code.data[vm->ip+2] << 4  |
                            vm->code.data[vm->ip+3];
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        LOG("JCOND %ld\n", pos);
        if(value_is_true(&val)) vm->ip = pos;
        else vm->ip += 4;
        dispatch();
    }
    doop(OP_JNCOND): { // jcond [64-bit position]
        vm->ip++;
        const uint32_t pos = vm->code.data[vm->ip+0] << 12 |
                             vm->code.data[vm->ip+1] << 8  |
                             vm->code.data[vm->ip+2] << 4  |
                             vm->code.data[vm->ip+3];
        struct value val = array_top(vm->stack);
        array_pop(vm->stack);
        LOG("JNCOND %ld\n", pos);
        if(!value_is_true(&val)) vm->ip = pos;
        else vm->ip += 4;
        dispatch();
    }
    doop(OP_CALL): {
        // argument: [arg2][arg1]
        vm->ip++;
        struct value val = array_top(vm->stack);
        int nargs = vm->code.data[vm->ip++];
        assert(vm->stack.length >= nargs);
        LOG("call %d\n", nargs);
        if(val.type == TYPE_NATIVE_FN) {
            array_pop(vm->stack);
            val.as.fn(vm, nargs);
        } else if(val.type == TYPE_FN || val.type == TYPE_DICT) {
            size_t fn_ip = 0;
            if(val.type == TYPE_DICT) {
                array_pop(vm->stack);
                struct value *ctor = dict_get(val.as.dict, "constructor");
                if(ctor == NULL) {
                    FATAL("expected dictionary to have constructor\n");
                    return;
                }
                if(ctor->type == TYPE_NATIVE_FN) {
                    value_free(&val);
                    ctor->as.fn(vm, nargs);
                    dispatch();
                } else if(ctor->type != TYPE_FN) {
                    FATAL("constructor must be a function!\n");
                    return;
                }
                fn_ip = ctor->as.ifn.ip;
                if(nargs+1 != ctor->as.ifn.nargs) {
                    FATAL("constructor expects exactly %d arguments, got %d\n", ctor->as.ifn.nargs, nargs+1);
                    return;
                }
            } else {
                fn_ip = val.as.ifn.ip;
                array_pop(vm->stack);
                if(nargs != val.as.ifn.nargs) {
                    FATAL("function expects exactly %d arguments, got %d\n", val.as.ifn.nargs, nargs);
                    return;
                }
            }
            struct value args[nargs];
            for(int i = nargs-1; i >= 0; i--) {
                struct value val = array_top(vm->stack);
                array_pop(vm->stack);
                args[i] = val;
            }
            // caller
            //struct value caller;
            //value_function(&caller, vm->ip, 0);
            //array_push(vm->stack, caller);
            struct env *oldenv = vm->localenv;
            vm->localenv = calloc(1, sizeof(struct env));
            vm->localenv->parent = oldenv;
            vm->localenv->ifn = vm->ip;
            // arguments
            if(val.type == TYPE_DICT) {
                if(vm->stack.length+nargs > vm->stack.capacity) {
                    vm->stack.capacity = vm->stack.length+nargs;
                    vm->stack.data = (struct value*)realloc(vm->stack.data,
                                                    sizeof(struct value)*vm->stack.capacity);
                }
                memcpy(vm->stack.data+vm->stack.length, args, sizeof(struct value)*nargs);
                vm->stack.length += nargs;

                struct value new_val;
                value_dict(&new_val);
                dict_set(new_val.as.dict, "prototype", &val);
                value_free(&val); // reference carried by dict
                array_push(vm->stack, new_val);
            } else {
                for(int i = 0; i < nargs; i++)
                    array_push(vm->stack, args[i]);
            }
            // jump
            vm->ip = fn_ip;
        } else {
            printf("is not a function\n");
            return;
        }
        dispatch();
    }
    doop(OP_RET): {
        LOG("RET\n");

        struct env *parent = vm->localenv->parent;
        env_free(vm->localenv);

        if(vm->localenv->ifn == (uint32_t)-1) {
            free(vm->localenv);
            vm->localenv = parent;
            return;
        } else {
            vm->ip = vm->localenv->ifn;
            free(vm->localenv);
            vm->localenv = parent;
        }
        dispatch();
    }

    // dictionaries
    doop(OP_DICT_NEW): {
        vm->ip++;
        LOG("DICT_NEW\n");
        array_push(vm->stack, (struct value){});
        value_dict(&array_top(vm->stack));
        dispatch();
    }
    doop(OP_MEMBER_GET):
    doop(OP_MEMBER_GET_NO_POP): {
        const enum vm_opcode op = vm->code.data[vm->ip];
        vm->ip++;
        char *key = (char *)&vm->code.data[vm->ip]; // must be null terminated
        vm->ip += strlen(key)+1;
        const uint32_t hash =
                            vm->code.data[vm->ip+0] << 12 |
                            vm->code.data[vm->ip+1] << 8  |
                            vm->code.data[vm->ip+2] << 4  |
                            vm->code.data[vm->ip+3];
        vm->ip+=sizeof(hash);
        LOG(op == OP_MEMBER_GET ? "MEMBER_GET %s\n" : "MEMBER_GET_NO_POP %s\n", key);

        struct value val = array_top(vm->stack);
        struct dict *dict = NULL;
        if(val.type != TYPE_DICT) {
            if(val.type == TYPE_STR) {
                dict = vm->dstr;
            } else if(val.type == TYPE_INT) {
                dict = vm->dint;
            } else if(val.type == TYPE_FLOAT) {
                dict = vm->dfloat;
            } else if(val.type == TYPE_ARRAY) {
                dict = vm->darray;
            } else if(val.type == TYPE_NIL) {
                if(strcmp(key, "prototype") != 0)
                    FATAL("can't access key of nil");
                dispatch();
            } else {
                FATAL("expected dictionary\n");
                return;
            }
            if(strcmp(key, "prototype") == 0) {
                value_free(&array_top(vm->stack));
                value_dict_copy(&array_top(vm->stack), dict);
                dispatch();
            }
        } else {
            if(val.type != TYPE_DICT) {
                FATAL("expected dictionary\n");
                return;
            }
            dict = val.as.dict;
            if(op == OP_MEMBER_GET) array_pop(vm->stack);
        }

        array_push(vm->stack, (struct value){});
        struct value *result = dict_get_hash(dict, key, hash);
        if(result != NULL)
            value_copy(&array_top(vm->stack), result);

        if(op == OP_MEMBER_GET) {
            value_free(&val);
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
        assert(dval.type == TYPE_DICT);
        array_pop(vm->stack);

        struct value val = array_top(vm->stack);
        LOG("SECOND %s\n", key);
        dict_set(dval.as.dict, key, &val);
        value_free(&dval);
        // NOTE: val should not be free'd.
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
            assert(key.type == TYPE_STR);
            array_pop(vm->stack); // pop val
            struct value val = array_top(vm->stack);
            array_pop(vm->stack);
            dict_set(dval.as.dict, string_data(key.as.str), &val);
            // pop key
            value_free(&val);
            value_free(&key);
        }
        array_pop(vm->stack); // pop nil
        array_push(vm->stack, dval);
        dispatch();
    }
    // array
    doop(OP_INDEX_GET): {
        vm->ip++;
        struct value index;
        value_copy(&index, &array_top(vm->stack));
        value_free(&array_top(vm->stack));
        array_pop(vm->stack);

        struct value dval = array_top(vm->stack);
        array_pop(vm->stack);

        if(dval.type == TYPE_ARRAY) {
            if(index.type != TYPE_INT) {
                FATAL("index type must be integer!\n");
                return;
            }
            const int64_t i = index.as.integer;
            if(!(i >= 0 && i < dval.as.array->data.length)) {
                FATAL("accessing index (%ld) that lies out of range [0,%ld) \n", i, dval.as.array->data.length);
                return;
            }
            array_push(vm->stack, (struct value){});
            value_copy(&array_top(vm->stack), &dval.as.array->data.data[i]);
        } else if(dval.type == TYPE_STR) {
            if(index.type != TYPE_INT) {
                FATAL("index type must be integer!\n");
                return;
            }
            const int64_t i = index.as.integer, len = string_len(dval.as.str);
            if(!(i >= 0 && i < len)) {
                FATAL("accessing index (%ld) that lies out of range [0,%ld) \n", i, len);
                return;
            }
            char c[2] = { string_at(dval.as.str, i), 0 };
            array_push(vm->stack, (struct value){});
            value_str(&array_top(vm->stack), c);
        } else if(dval.type == TYPE_DICT) {
            if(index.type != TYPE_STR) {
                FATAL("index type must be string!\n");
                return;
            }
            array_push(vm->stack, (struct value){});
            struct value *val = dict_get(dval.as.dict, string_data(index.as.str));
            value_free(&index);
            if(val != NULL) value_copy(&array_top(vm->stack), val);
        } else {
            FATAL("expected dictionary, array or string\n");
            return;
        }
        value_free(&dval);
        dispatch();
    }
    doop(OP_INDEX_SET): {
        vm->ip++;
        LOG("INDEX_SET\n");

        struct value index;
        value_copy(&index, &array_top(vm->stack));
        value_free(&array_top(vm->stack));
        array_pop(vm->stack);

        struct value dval = array_top(vm->stack);
        array_pop(vm->stack);

        struct value *val = &array_top(vm->stack);

        if(dval.type == TYPE_ARRAY) {
            if(index.type != TYPE_INT) {
                value_free(&index);
                FATAL("index type must be integer!\n");
                return;
            }
            const int64_t i = index.as.integer;
            if(!(i >= 0 && i < dval.as.array->data.length)) {
                FATAL("accessing index (%ld) that lies out of range [0,%ld) \n", i, dval.as.array->data.length);
                return;
            }
            value_free(&dval.as.array->data.data[i]);
            value_copy(&dval.as.array->data.data[i], val);
        } else if(dval.type == TYPE_DICT) {
            if(index.type != TYPE_STR) {
                FATAL("index type must be string!\n");
                return;
            }
            dict_set(dval.as.dict, string_data(index.as.str), val);
            value_free(&index);
        } else {
            FATAL("expected dictionary or array\n");
            return;
        }
        value_free(&dval);
        dispatch();
    }
    doop(OP_ARRAY_LOAD): {
        vm->ip++;

        struct value val = array_top(vm->stack);
        int length = val.as.integer;
        array_pop(vm->stack);
        assert(val.type == TYPE_INT);
        LOG("ARRAY_LOAD %d\n", length);

        struct value aval;
        if(length == 0) {
            value_array(&aval);
        } else {
            value_array_n(&aval, length);
            aval.as.array->data.length = length;
            while(length--) {
                struct value val = array_top(vm->stack);
                value_copy(&aval.as.array->data.data[length], &val);
                array_pop(vm->stack);
                value_free(&val);
            }
        }
        array_push(vm->stack, aval);
        dispatch();
    }

    }
}

struct value *vm_call(struct vm *vm, struct value *fn, a_arguments args) {
    assert(fn->type == TYPE_FN || fn->type == TYPE_DICT);

    uint32_t nargs = 0, ip = 0;
    if(fn->type == TYPE_DICT) {
        struct value *ctor = dict_get(fn->as.dict, "constructor");
        if(ctor == NULL) {
            printf("expected dictionary to have constructor");
            return NULL;
        }
        if(ctor->type == TYPE_NATIVE_FN) {
            for(int64_t i = args.length-1; i >= 0; i--) {
                array_push(vm->stack, (struct value){0});
                value_copy(&array_top(vm->stack), &args.data[i]);
            }
            ctor->as.fn(vm, args.length);
            return &array_top(vm->stack);
        } else if(ctor->type != TYPE_FN) {
            printf("constructor must be a function!\n");
            return NULL;
        } else {
            nargs = ctor->as.ifn.nargs;
            ip = ctor->as.ifn.ip;
        }
    } else if(fn->type == TYPE_FN) {
        nargs = fn->as.ifn.nargs;
        ip = fn->as.ifn.ip;
    }

    if((uint32_t)args.length != nargs) {
        printf("function requires %d arguments, got %ld\n", nargs, args.length);
        return NULL;
    }
    const uint32_t last = vm->ip;
    // setup env
    struct env *oldenv = vm->localenv;
    vm->localenv = calloc(1, sizeof(struct env));
    vm->localenv->parent = oldenv;
    vm->localenv->ifn = (uint32_t)-1;
    // setup stack/ip
    for(int64_t i = args.length-1; i >= 0; i--) {
        array_push(vm->stack, (struct value){0});
        value_copy(&array_top(vm->stack), &args.data[i]);
    }
    vm->ip = ip;
    // environment (already set up by body)
    // call it
    vm_execute(vm);
    // restore env
    vm->ip = last;
    return &array_top(vm->stack);
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

void vm_code_reserve(struct vm *vm, size_t s) {
    free(vm->code.data);
    vm->code.data = malloc(s);
    vm->code.length = 0;
}
