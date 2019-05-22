#include <stdio.h>
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
void value_function(struct value *val, uint32_t ip, uint16_t nargs, struct env *env) {
    val->type = TYPE_FN;
    val->as.ifn = function_malloc(ip, nargs, env);
}
void value_dict(struct value *val) {
    val->type = TYPE_DICT;
    val->as.dict = dict_malloc();
}
void value_array(struct value *val) {
    val->type = TYPE_ARRAY;
    val->as.array = array_obj_malloc();
}
void value_array_n(struct value *val, size_t n) {
    val->type = TYPE_ARRAY;
    val->as.array = array_obj_malloc_n(n);
}

// arith
#define arith_op(name, op, custom) \
void value_ ## name (struct value *result, const struct value left, const struct value right) { \
    switch(left.type) { \
    case TYPE_INT: { \
        switch(right.type) { \
        case TYPE_FLOAT: { \
            value_float(result, (double)left.as.integer op right.as.floatp); break; } \
        case TYPE_INT: { \
            value_int(result, left.as.integer op right.as.integer); \
            break; }\
        default: \
            result->type = TYPE_INTERPRETER_ERROR; } \
        break; \
    } \
    case TYPE_FLOAT: { \
        switch(right.type) { \
        case TYPE_INT: { \
            value_float(result, left.as.floatp op (double)right.as.integer); break; } \
        case TYPE_FLOAT: { \
            value_float(result, left.as.floatp op right.as.floatp); break; } \
        default: \
            result->type = TYPE_INTERPRETER_ERROR; } \
        break; \
    } custom \
    default: result->type = TYPE_INTERPRETER_ERROR; }\
}
arith_op(add, +,
    case TYPE_STR: {
        if(right.type != TYPE_STR) {
            result->type = TYPE_INTERPRETER_ERROR;
            return;
        }
        result->type = TYPE_STR;
        result->as.str = string_append(left.as.str, right.as.str);
        break; }
)
arith_op(sub, -,)
arith_op(mul, *,
    case TYPE_STR: {
        if(right.type == TYPE_INT) {
            if(right.as.integer == 0) {
                value_str(result, "");
            } else {
                result->type = TYPE_STR;
                result->as.str = string_repeat(left.as.str, right.as.integer);
            }
        }
        else
            result->type = TYPE_INTERPRETER_ERROR;
        break; }
    case TYPE_ARRAY: {
        if(right.type == TYPE_INT) {
            result->type = TYPE_ARRAY;
            result->as.array = array_obj_repeat(left.as.array, (size_t)right.as.integer);
        }
        else
            result->type = TYPE_INTERPRETER_ERROR;
        break; }
)
void value_div(struct value *result, const struct value left, const struct value right) {
    switch(left.type) {
    case TYPE_INT: {
        switch(right.type) {
        case TYPE_FLOAT: {
            value_float(result, (double)left.as.integer / right.as.floatp); break; }
        case TYPE_INT: {
            value_int(result, (double)left.as.integer / (double)right.as.integer);
            break; }
        default:
            result->type = TYPE_INTERPRETER_ERROR; }
        break;
    }
    case TYPE_FLOAT: {
        switch(right.type) {
        case TYPE_INT: {
            value_float(result, left.as.floatp / (double)right.as.integer); break; }
        case TYPE_FLOAT: {
            value_float(result, left.as.floatp / right.as.floatp); break; }
        default:
            result->type = TYPE_INTERPRETER_ERROR; }
        break;
    }
    default: result->type = TYPE_INTERPRETER_ERROR; }
}
void value_mod(struct value *result, const struct value left, const struct value right) {
    if(left.type == TYPE_INT && right.type == TYPE_INT) {
        value_int(result, left.as.integer % right.as.integer);
    } else
        result->type = TYPE_INTERPRETER_ERROR;
}

// in place
// returns 1 if it CAN do it in place
int value_iadd(struct value left, const struct value right) {
    switch (left.type) {
        case TYPE_STR: {
            switch(right.type) {
            case TYPE_STR: {
                string_append_in_place(left.as.str, right.as.str);
                return 1; } }
            return 0;
        }
    }
    return 0;
}

int value_imul(struct value left, const struct value right) {
    switch (left.type) {
        case TYPE_STR: {
            switch(right.type) {
            case TYPE_INT: {
                string_repeat_in_place(left.as.str, right.as.integer);
                return 1; } }
            return 0;
        }
    }
    return 0;
}

// comparison
#define strcmp_op(cond) \
    case TYPE_STR: \
        value_int(result, right.type == TYPE_STR && string_cmp(left.as.str, right.as.str) cond); \
        break;
arith_op(eq, ==,
    strcmp_op(== 0)
    case TYPE_NATIVE_FN:
    case TYPE_FN:
    case TYPE_DICT:
        value_int(result, left.as.integer == right.as.integer);
        break;
    case TYPE_NIL:
        value_int(result, right.type == TYPE_NIL);
        break;
)
arith_op(neq, !=,
    strcmp_op(!= 0)
    case TYPE_NATIVE_FN:
    case TYPE_FN:
    case TYPE_DICT:
        value_int(result, left.as.integer != right.as.integer);
        break;
    case TYPE_NIL:
        value_int(result, right.type != TYPE_NIL);
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
bool value_is_true(const struct value val) {
    switch(val.type) {
    case TYPE_INT: return val.as.integer > 0;
    case TYPE_FLOAT: return val.as.floatp > 0;
    case TYPE_STR: return !string_is_empty(val.as.str);
    default: return 0;
    }
}

struct dict *value_get_prototype(const struct vm *vm, const struct value val) {
    if(val.type == TYPE_STR) {
        return vm->dstr;
    } else if(val.type == TYPE_INT) {
        return vm->dint;
    } else if(val.type == TYPE_FLOAT) {
        return vm->dfloat;
    } else if(val.type == TYPE_ARRAY) {
        return vm->darray;
    } else if(val.type == TYPE_DICT) {
        const struct value *p = dict_get(val.as.dict, "prototype");
        if (p == NULL) return NULL;
        return p->as.dict;
    }
    return NULL;
}
