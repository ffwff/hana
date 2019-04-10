#pragma once
#include <stdint.h>

struct value {
    union {
        int integer;
        float floatp;
        char *str;
    } as;
    enum {
        TYPE_NIL, TYPE_INT, TYPE_FLOAT, TYPE_STR
    } type;
};

void value_int(struct value*, int);
void value_float(struct value*, float);
void value_str(struct value*, char*);
void value_free(struct value*);
void value_print(struct value*);
void value_copy(struct value *left, struct value *right);

void value_add(struct value *result, struct value *left, struct value *right);
void value_sub(struct value *result, struct value *left, struct value *right);
void value_mul(struct value *result, struct value *left, struct value *right);
void value_div(struct value *result, struct value *left, struct value *right);
void value_mod(struct value *result, struct value *left, struct value *right);
