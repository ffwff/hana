#include <stdio.h>
#include <assert.h>
#include <stdlib.h>
#include "string_.h"
#include "value.h"
#include "vm.h"
#include "dict.h"
#include "array_obj.h"
#include "function.h"

void value_int(struct value *val, int64_t data) {
    val->type = TYPE_INT;
    val->as.integer = data;
}
void value_float(struct value *val, double data) {
    val->type = TYPE_FLOAT;
    val->as.floatp = data;
}
// non-primitives
void value_str(struct value *val, const char *data) {
    val->type = TYPE_STR;
    val->as.str = malloc(string_size(data));
    string_init(val->as.str, data);
}
void value_str_reserve(struct value *val, const size_t size) {
    val->type = TYPE_STR;
    val->as.str = malloc(sizeof(struct string_header)+size+1);
    val->as.str->length = size;
}
void value_strmov(struct value *val, struct string_header *str) {
    val->type = TYPE_STR;
    val->as.str = str;
}
void value_native(struct value *val, value_fn fn) {
    val->type = TYPE_NATIVE_FN;
    val->as.fn = fn;
}
void value_function(struct value *val, uint32_t ip, uint16_t nargs, struct env *env) {
    val->type = TYPE_FN;
    val->as.ifn = malloc(sizeof(struct function));
    function_init(val->as.ifn, ip, nargs, env);
    //GC_register_finalizer(val->as.ifn, (GC_finalization_proc)function_free, NULL, NULL, NULL);
}
/*
void value_dict(struct value *val) {
    val->type = TYPE_DICT;
    val->as.dict = malloc(sizeof(struct dict));
    dict_init(val->as.dict);
    GC_register_finalizer(val->as.dict, (GC_finalization_proc)dict_free, NULL, NULL, NULL);
}
*/
void value_array(struct value *val) {
    val->type = TYPE_ARRAY;
    val->as.array = malloc(sizeof(struct array_obj));
    array_obj_init(val->as.array);
    //GC_register_finalizer(val->as.array, (GC_finalization_proc)array_obj_free, NULL, NULL, NULL);
}
void value_array_n(struct value *val, size_t n) {
    val->type = TYPE_ARRAY;
    val->as.array = malloc(sizeof(struct array_obj));
    array_obj_init_n(val->as.array, n);
    //GC_register_finalizer(val->as.array, (GC_finalization_proc)array_obj_free, NULL, NULL, NULL);
}
void value_native_obj(struct value *val, void *data, native_obj_free_fn free) {
    val->type = TYPE_NATIVE_OBJ;
    val->as.native = malloc(sizeof(struct native_obj));
    native_obj_init(val->as.native, data, free);
    //GC_register_finalizer(val->as.native, (GC_finalization_proc)native_obj_free, NULL, NULL, NULL);
}

void value_print(struct value *val) {
    if(val->type == TYPE_INT)
        printf("%ld", val->as.integer);
    else if(val->type == TYPE_FLOAT)
        printf("%f", val->as.floatp);
    else if(val->type == TYPE_STR)
        printf("\"%s\"", string_data(val->as.str));
    else if(val->type == TYPE_NATIVE_FN)
        printf("[native fn %lx]", (intptr_t)val->as.fn);
    else if(val->type == TYPE_FN)
        printf("[fn %d]", (uint32_t)val->as.ifn->ip);
    else if(val->type == TYPE_DICT)
        printf("[dict %p]", val->as.dict);
    else if(val->type == TYPE_ARRAY)
        printf("[array %p]", val->as.array);
    else if(val->type == TYPE_NATIVE_OBJ)
        printf("[native obj %p]", val->as.native);
    else {
        printf("nil");
    }
}

void value_copy(struct value *dst, const struct value *src) {
    dst->type = src->type;
    dst->as = src->as;
}

// arith
#define arith_op(name, op, custom) \
void value_ ## name (struct value *result, const struct value *left, const struct value *right) { \
    switch(left->type) { \
    case TYPE_INT: { \
        if(right->type == TYPE_FLOAT) \
            value_float(result, (double)left->as.integer op right->as.floatp); \
        else if(right->type == TYPE_INT) \
            value_int(result, left->as.integer op right->as.integer); \
        else \
            result->type = TYPE_INTERPRETER_ERROR; \
        break; \
    } \
    case TYPE_FLOAT: { \
        if(right->type == TYPE_INT) \
            value_float(result, left->as.floatp op (double)right->as.integer); \
        else if(right->type == TYPE_FLOAT) \
            value_float(result, left->as.floatp op right->as.floatp); \
        else \
            result->type = TYPE_INTERPRETER_ERROR; \
        break; \
    } custom \
    default: result->type = TYPE_INTERPRETER_ERROR; }\
}
arith_op(add, +,
    case TYPE_STR: {
        if(right->type != TYPE_STR) {
            result->type = TYPE_INTERPRETER_ERROR;
            return;
        }
        struct string_header *s = string_alloc(string_len(left->as.str)+string_len(right->as.str));
        char *ss = string_data(s); ss[0] = 0;
        strcpy(ss, string_data(left->as.str));
        strcpy(ss+string_len(left->as.str), string_data(right->as.str));
        result->type = TYPE_STR;
        result->as.str = s;
        break; }
)
arith_op(sub, -,)
arith_op(mul, *,
    case TYPE_STR: {
        if(right->type == TYPE_INT) {
            if(right->as.integer == 0) {
                value_str(result, "");
            } else {
                size_t n = string_len(left->as.str)*right->as.integer;
                struct string_header *s = string_alloc(n);
                char *ss = string_data(s); ss[0] = 0;
                for(size_t i = 0; i < (size_t)right->as.integer; i++)
                    strcat(ss, string_data(left->as.str));
                result->type = TYPE_STR;
                result->as.str = s;
            }
        }
        else
            result->type = TYPE_INTERPRETER_ERROR;
        break; }
    case TYPE_ARRAY: {
        if(right->type == TYPE_INT) {
            size_t length = left->as.array->data.length*(size_t)right->as.integer;
            value_array_n(result, length);
            for (size_t i = 0; i < (size_t)right->as.integer; i++)
            {
                for(size_t j = 0; j < left->as.array->data.length; j++) {
                    size_t index = left->as.array->data.length*i+j;
                    value_copy(&result->as.array->data.data[index], &left->as.array->data.data[j]);
                }
            }
        }
        else
            result->type = TYPE_INTERPRETER_ERROR;
        break; }
)
void value_div(struct value *result, const struct value *left, const struct value *right) {
    if( (left->type == TYPE_FLOAT && right->type == TYPE_INT) ||
        (left->type == TYPE_INT && right->type == TYPE_FLOAT) ) {
        const struct value *floatv = left->type == TYPE_FLOAT ? left : right;
        const struct value *intv = left->type == TYPE_INT ? left : right;
        value_float(result, floatv->as.floatp / (double)intv->as.integer);
    } else if(left->type == TYPE_INT && right->type == TYPE_INT) {
        value_float(result, (double)left->as.integer / (double)right->as.integer);
    } else if(left->type == TYPE_FLOAT && right->type == TYPE_FLOAT) {
        value_float(result, left->as.floatp / right->as.floatp);
    } else
        result->type = TYPE_INTERPRETER_ERROR;
}
void value_mod(struct value *result, const struct value *left, const struct value *right) {
    if(left->type == TYPE_INT && right->type == TYPE_INT) {
        value_int(result, left->as.integer % right->as.integer);
    } else
        result->type = TYPE_INTERPRETER_ERROR;
}

// logic
#define logic_op(name, op) \
void value_ ## name (struct value *result, const struct value *left, const struct value *right) { \
    value_int(result, value_is_true(left) op value_is_true(right)); \
}
logic_op(and, &&)
logic_op(or, ||)

// comparison
#define strcmp_op(cond) \
    case TYPE_STR: \
        value_int(result, right->type == TYPE_STR && string_cmp(left->as.str, right->as.str) cond); \
        break;
arith_op(eq, ==,
    strcmp_op(== 0)
    case TYPE_NATIVE_FN:
    case TYPE_FN:
    case TYPE_DICT:
        value_int(result, left->as.integer == right->as.integer);
        break;
    case TYPE_NIL:
        value_int(result, right->type == TYPE_NIL);
        break;
)
arith_op(neq, !=,
    strcmp_op(!= 0)
    case TYPE_NATIVE_FN:
    case TYPE_FN:
    case TYPE_DICT:
        value_int(result, left->as.integer != right->as.integer);
        break;
    case TYPE_NIL:
        value_int(result, right->type != TYPE_NIL);
        break;
)
arith_op(lt, <,
    strcmp_op(< 0)
)
arith_op(leq, <=,
    strcmp_op(<= 0)
)
arith_op(gt, >,
    strcmp_op(> 0)
)
arith_op(geq, >=,
    strcmp_op(>= 0)
)

// boolean
bool value_is_true(const struct value *val) {
    switch(val->type) {
    case TYPE_INT: return val->as.integer > 0;
    case TYPE_FLOAT: return val->as.floatp > 0;
    case TYPE_STR: return string_len(val->as.str) > 0;
    default: return 0;
    }
}

struct dict *value_get_prototype(const struct vm *vm, const struct value *val) {
    if(val->type == TYPE_STR) {
        return vm->dstr;
    } else if(val->type == TYPE_INT) {
        return vm->dint;
    } else if(val->type == TYPE_FLOAT) {
        return vm->dfloat;
    } else if(val->type == TYPE_ARRAY) {
        return vm->darray;
    } else if(val->type == TYPE_DICT) {
        return dict_get_prototype(val->as.dict);
    }
    return NULL;
}