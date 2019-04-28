#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include <stdint.h>
#include "native_obj.h"

struct vm;
struct dict;
struct array_obj;
struct string_header;

typedef void (*value_fn)(struct vm *vm, int nargs);
struct value {
    union {
        int64_t integer;
        double floatp;
        struct string_header *str;
        value_fn fn;
        struct function *ifn;
        struct dict *dict;
        struct array_obj *array;
        struct native_obj *native;
        void *ptr;
    } as;
    enum value_type {
        TYPE_NIL, TYPE_INT, TYPE_FLOAT,
        TYPE_NATIVE_FN, TYPE_FN,
        TYPE_STR, TYPE_DICT, TYPE_ARRAY,
        TYPE_NATIVE_OBJ
    } type;
};

void value_int(struct value*, int64_t);
void value_float(struct value*, double);
void value_str(struct value*, const char*);
void value_str_reserve(struct value*, size_t);
void value_strmov(struct value*, struct string_header*);
void value_native(struct value*, value_fn);
struct env;
void value_function(struct value*, uint32_t ip, uint16_t nargs, struct env *env);
void value_dict(struct value*);
void value_dict_copy(struct value*, struct dict*);
void value_dict_copy_noref(struct value*, const struct dict*);
void value_array(struct value*);
void value_array_n(struct value*, size_t n);
void value_native_obj(struct value*, void *data, native_obj_free_fn free);

void value_print(struct value*);

void value_free(struct value *src);
void value_copy(struct value *dst, struct value *src);

void value_add(struct value *result, const struct value *left, const struct value *right);
void value_sub(struct value *result, const struct value *left, const struct value *right);
void value_mul(struct value *result, const struct value *left, const struct value *right);
void value_div(struct value *result, const struct value *left, const struct value *right);
void value_mod(struct value *result, const struct value *left, const struct value *right);

void value_and(struct value *result, const struct value *left, const struct value *right);
void value_or(struct value *result, const struct value *left, const struct value *right);

void value_lt(struct value *result, const struct value *left, const struct value *right);
void value_leq(struct value *result, const struct value *left, const struct value *right);
void value_gt(struct value *result, const struct value *left, const struct value *right);
void value_geq(struct value *result, const struct value *left, const struct value *right);
void value_eq(struct value *result, const struct value *left, const struct value *right);
void value_neq(struct value *result, const struct value *left, const struct value *right);

int value_is_true(const struct value *);
struct dict *value_get_prototype(const struct vm *vm, const struct value *val);

#ifdef __cplusplus
}
#endif
