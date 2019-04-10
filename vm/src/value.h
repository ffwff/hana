#pragma once
#include <stdint.h>

struct value {
    union {
        int integer;
        float floatp;
    } as;
    enum {
        TYPE_INT, TYPE_FLOAT
    } type;
};

void value_int(struct value*, int);
void value_free(struct value*);
void value_print(const struct value*);
void value_copy(struct value *left, struct value *right);

void value_add(struct value *result, const struct value *left, const struct value *right);
void value_sub(struct value *result, const struct value *left, const struct value *right);
void value_mul(struct value *result, const struct value *left, const struct value *right);
void value_div(struct value *result, const struct value *left, const struct value *right);
void value_mod(struct value *result, const struct value *left, const struct value *right);
