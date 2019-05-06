#include <stdio.h>
#include <assert.h>
#include <stdlib.h>
#include "string_.h"
#include "value.h"
#include "vm.h"
#include "hmap.h"
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
    val->as.str = string_malloc(data);
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
void value_dict(struct value *val) {
    val->type = TYPE_DICT;
    val->as.dict = hmap_malloc();
    //GC_register_finalizer(val->as.dict, (GC_finalization_proc)dict_free, NULL, NULL, NULL);
}
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
        printf("[string]");
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
        result->type = TYPE_STR;
        result->as.str = string_append(left->as.str, right->as.str);
        break; }
)
arith_op(sub, -,)
arith_op(mul, *,
    case TYPE_STR: {
        if(right->type == TYPE_INT) {
            if(right->as.integer == 0) {
                value_str(result, "");
            } else {
                result->type = TYPE_STR;
                result->as.str = string_repeat(left->as.str, right->as.integer);
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
    case TYPE_STR: return !string_is_empty(val->as.str);
    default: return 0;
    }
}

struct hmap *value_get_prototype(const struct vm *vm, const struct value *val) {
    if(val->type == TYPE_STR) {
        return vm->dstr;
    } else if(val->type == TYPE_INT) {
        return vm->dint;
    } else if(val->type == TYPE_FLOAT) {
        return vm->dfloat;
    } else if(val->type == TYPE_ARRAY) {
        return vm->darray;
    } else if(val->type == TYPE_DICT) {
        return hmap_get(val->as.dict, "prototype")->as.dict;
    }
    return NULL;
}
