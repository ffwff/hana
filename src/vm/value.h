#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include <stdint.h>
#include <stdbool.h>
#include "native_obj.h"
#include "dict.h"
#include "array_obj.h"

struct vm;
struct hmap;
struct array_obj;
struct string;

#define TYPE_NIL        0
#define TYPE_INT        1
#define TYPE_FLOAT      2
#define TYPE_NATIVE_FN  3
#define TYPE_FN         4
#define TYPE_STR        5
#define TYPE_DICT       6
#define TYPE_ARRAY      7
#define TYPE_NATIVE_OBJ 8
#define TYPE_INTERPRETER_ERROR 127

typedef void (*value_fn)(struct vm *vm, int nargs);
struct __attribute__((packed)) value
{
    // memory efficient packed data structure for storing
    // dynamically-typed values (patent pending)
    union {
        int64_t integer;
        double floatp;
        struct string *str;
        value_fn fn;
        struct function *ifn;
        struct dict *dict;
        array_obj *array;
        struct native_obj *native;
    } as;
    uint8_t type;
};

void value_int(struct value*, int64_t);
void value_float(struct value*, double);
void value_str(struct value*, const char*);
void value_native(struct value*, value_fn);
struct env;
void value_function(struct value*, uint32_t ip, uint16_t nargs, struct env *env);
void value_dict(struct value*);
void value_array(struct value*);
void value_array_n(struct value*, size_t n);
void value_native_obj(struct value*, void *data, native_obj_free_fn free);

void value_print(struct value*);

void value_add(struct value *result, const struct value left, const struct value right);
void value_sub(struct value *result, const struct value left, const struct value right);
void value_mul(struct value *result, const struct value left, const struct value right);
void value_div(struct value *result, const struct value left, const struct value right);
void value_mod(struct value *result, const struct value left, const struct value right);

void value_and(struct value *result, const struct value left, const struct value right);
void value_or(struct value  *result, const struct value left, const struct value right);

void value_lt(struct value  *result, const struct value left, const struct value right);
void value_leq(struct value *result, const struct value left, const struct value right);
void value_gt(struct value  *result, const struct value left, const struct value right);
void value_geq(struct value *result, const struct value left, const struct value right);
void value_eq(struct value  *result, const struct value left, const struct value right);
void value_neq(struct value *result, const struct value left, const struct value right);

bool value_is_true(const struct value);
struct dict *value_get_prototype(const struct vm *vm, const struct value val);

#ifdef __cplusplus
}
#endif
