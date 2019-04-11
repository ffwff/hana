#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include <stdint.h>

struct vm;
typedef void (*value_fn)(struct vm *vm, int nargs);
struct value {
    union {
        int integer;
        float floatp;
        char *str;
        value_fn fn;
    } as;
    enum {
        TYPE_NIL, TYPE_INT, TYPE_FLOAT, TYPE_STR,
        TYPE_NATIVE_FN
    } type;
};

void value_int(struct value*, int);
void value_float(struct value*, float);
void value_str(struct value*, const char*);
void value_function(struct value*, value_fn);

void value_free(struct value*);
void value_print(struct value*);
void value_copy(struct value *left, struct value *right);

void value_add(struct value *result, struct value *left, struct value *right);
void value_sub(struct value *result, struct value *left, struct value *right);
void value_mul(struct value *result, struct value *left, struct value *right);
void value_div(struct value *result, struct value *left, struct value *right);
void value_mod(struct value *result, struct value *left, struct value *right);

void value_and(struct value *result, struct value *left, struct value *right);
void value_or(struct value *result, struct value *left, struct value *right);

void value_lt(struct value *result, struct value *left, struct value *right);
void value_leq(struct value *result, struct value *left, struct value *right);
void value_gt(struct value *result, struct value *left, struct value *right);
void value_geq(struct value *result, struct value *left, struct value *right);
void value_eq(struct value *result, struct value *left, struct value *right);
void value_neq(struct value *result, struct value *left, struct value *right);

int value_is_true(const struct value *);

#ifdef __cplusplus
}
#endif
