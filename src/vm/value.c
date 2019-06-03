#include <stdio.h>
#include <stdlib.h>
#include "string_.h"
#include "value.h"
#include "vm.h"
#include "hmap.h"
#include "array_obj.h"
#include "function.h"

int value_get_type(struct value val) {
    if(val.as.bits.reserved_nan == RESERVED_NAN && val.as.bits.tag_bits > 0) {
        return val.as.bits.tag_bits;
    } else {
        return TYPE_FLOAT;
    }
}

// non-primitives
struct value value_int(int32_t n) {
    struct value val = {0};
    value_set_int(&val, n);
    return val;
}
struct value value_float(double n) {
    struct value val;
    value_set_float(&val, n);
    return val;
}
struct value value_str(const char *data, const struct vm *vm) {
    return value_pointer(TYPE_STR, string_malloc(data, vm));
}
struct value value_function(uint32_t ip, uint16_t nargs, struct env *env, const struct vm *vm) {
    return value_pointer(TYPE_FN, function_malloc(ip, nargs, env, vm));
}
struct value value_dict(const struct vm *vm) {
    return value_pointer(TYPE_DICT, dict_malloc(vm));
}
struct value value_dict_n(size_t n, const struct vm *vm) {
    return value_pointer(TYPE_DICT, dict_malloc_n(vm, n));
}
struct value value_array(const struct vm *vm) {
    return value_pointer(TYPE_ARRAY, array_obj_malloc(vm));
}
struct value value_array_n(size_t n, const struct vm *vm) {
    return value_pointer(TYPE_ARRAY, array_obj_malloc_n(n, vm));
}

struct value value_interpreter_error() {
    return value_pointer(TYPE_INTERPRETER_ERROR, 0);
}

// arith
#define arith_op(name, op, custom)                                                                      \
    struct value value_##name(const struct value left, const struct value right, const struct vm *vm) { \
        switch (value_get_type(left)) {                                                                 \
            case TYPE_INT: {                                                                            \
                switch (value_get_type(right)) {                                                        \
                    case TYPE_FLOAT:                                                                    \
                        return value_float((double)value_get_int(left) op right.as.floatp);             \
                    case TYPE_INT:                                                                      \
                        return value_int(value_get_int(left) op value_get_int(right));                  \
                    default:                                                                            \
                        return value_interpreter_error();                                               \
                }                                                                                       \
            }                                                                                           \
            case TYPE_FLOAT: {                                                                          \
                switch (value_get_type(right)) {                                                        \
                    case TYPE_INT:                                                                      \
                        return value_int(left.as.floatp op(double) value_get_int(right));               \
                    case TYPE_FLOAT:                                                                    \
                        return value_float(left.as.floatp op right.as.floatp);                          \
                    default:                                                                            \
                        return value_interpreter_error();                                               \
                }                                                                                       \
            }                                                                                           \
                custom                                                                                  \
        }                                                                                               \
        return value_interpreter_error();                                                               \
    }
arith_op(add, +,
    case TYPE_STR: {
        if(value_get_type(right) != TYPE_STR) {
            return value_interpreter_error();
        }
        return value_pointer(TYPE_STR, string_append(value_get_pointer(TYPE_STR, left), value_get_pointer(TYPE_STR, right), vm));
        }
)
arith_op(sub, -,)
arith_op(mul, *,
    case TYPE_STR: {
        if(value_get_type(right) == TYPE_INT) {
            if(value_get_int(right) == 0) {
                return value_interpreter_error();
            }
            return value_pointer(TYPE_STR, string_repeat(value_get_pointer(TYPE_STR, left), value_get_int(right), vm));
        }
        return value_interpreter_error();
    }
    case TYPE_ARRAY: {
        if(value_get_type(right) == TYPE_INT) {
            return value_pointer(TYPE_ARRAY, array_obj_repeat(value_get_pointer(TYPE_ARRAY, left), (size_t)value_get_int(right), vm));
        }
        return value_interpreter_error();
    }
)

struct value value_div(const struct value left, const struct value right, const struct vm *_) {
    switch(value_get_type(left)) {
    case TYPE_INT: {
        switch(value_get_type(right)) {
            case TYPE_FLOAT:
                return value_float((double)value_get_int(left) / right.as.floatp);
            case TYPE_INT:
                return value_float((double)value_get_int(left) / (double)value_get_int(right));
        }
        return value_interpreter_error();
    }
    case TYPE_FLOAT: {
        switch(value_get_type(right)) {
            case TYPE_INT:
                return value_float(left.as.floatp / (double)value_get_int(right));
            case TYPE_FLOAT:
                return value_float(left.as.floatp / right.as.floatp);
            }
        return value_interpreter_error();
    }
    default: return value_interpreter_error(); }
}
struct value value_mod(const struct value left, const struct value right, const struct vm *_) {
    if(value_get_type(left) == TYPE_INT && value_get_type(right) == TYPE_INT) {
        return value_int(value_get_int(left) % value_get_int(right));
    } else
        return value_interpreter_error();
}
struct value value_bitwise_and(const struct value left, const struct value right, const struct vm *_) {
    if (value_get_type(left) == TYPE_INT && value_get_type(right) == TYPE_INT) {
        return value_int(value_get_int(left) & value_get_int(right));
    } else
        return value_interpreter_error();
}
struct value value_bitwise_or(const struct value left, const struct value right, const struct vm *_) {
    if (value_get_type(left) == TYPE_INT && value_get_type(right) == TYPE_INT) {
        return value_int(value_get_int(left) | value_get_int(right));
    } else
        return value_interpreter_error();
}
struct value value_bitwise_xor(const struct value left, const struct value right, const struct vm *_) {
    if (value_get_type(left) == TYPE_INT && value_get_type(right) == TYPE_INT) {
        return value_int(value_get_int(left) ^ value_get_int(right));
    } else
        return value_interpreter_error();
}

// in place
// returns 1 if it CAN do it in place
int value_iadd(struct value left, const struct value right) {
    switch (value_get_type(left)) {
        case TYPE_STR: {
            switch (value_get_type(right)) {
                case TYPE_STR: {
                    string_append_in_place(value_get_pointer(TYPE_STR, left), value_get_pointer(TYPE_STR, right));
                    return 1;
                }
            }
            return 0;
        }
    }
    return 0;
}

int value_imul(struct value left, const struct value right) {
    switch (value_get_type(left)) {
        case TYPE_STR: {
            switch (value_get_type(right)) {
                case TYPE_INT: {
                    string_repeat_in_place(value_get_pointer(TYPE_STR, left), value_get_int(right));
                    return 1;
                }
            }
            return 0;
        }
    }
    return 0;
}

// comparison
#define strcmp_op(cond) \
    case TYPE_STR: \
        return value_int(value_get_type(right) == TYPE_STR && string_cmp(value_get_pointer(TYPE_STR, left), value_get_pointer(TYPE_STR, right)) cond); \
        break;
arith_op(eq, ==,
    strcmp_op(== 0)
    case TYPE_NATIVE_FN:
    case TYPE_FN:
    case TYPE_DICT:
        return value_int(value_get_int(left) == value_get_int(right));
    case TYPE_NIL:
        return value_int(value_get_type(right) == TYPE_NIL);
)
arith_op(neq, !=,
    strcmp_op(!= 0)
    case TYPE_NATIVE_FN:
    case TYPE_FN:
    case TYPE_DICT:
        return value_int(value_get_int(left) != value_get_int(right));
    case TYPE_NIL:
        return value_int(value_get_type(right) != TYPE_NIL);
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
    switch(value_get_type(val)) {
    case TYPE_INT: return value_get_int(val) > 0;
    case TYPE_FLOAT: return val.as.floatp > 0;
    case TYPE_STR: return !string_is_empty(value_get_pointer(TYPE_STR, val));
    default: return 0;
    }
}

struct dict *value_get_prototype(const struct vm *vm, const struct value val) {
    switch (value_get_type(val)) {
        case TYPE_STR:
            return vm->dstr;
        case TYPE_INT:
            return vm->dint;
        case TYPE_FLOAT:
            return vm->dfloat;
        case TYPE_ARRAY:
            return vm->darray;
        case TYPE_DICT: {
            const struct value *p = dict_get(value_get_pointer(TYPE_DICT, val), "prototype");
            if (p == NULL) return NULL;
            return value_get_pointer(TYPE_DICT, *p);
        }
    }
    return NULL;
}
