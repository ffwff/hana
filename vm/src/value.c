#include <string.h>
#include <stdio.h>
#include <assert.h>
#include "value.h"

void value_int(struct value *val, int data) { val->type = TYPE_INT; val->as.integer = data; }

void value_free(struct value *val) {

}

void value_print(const struct value *val) {
    if(val->type == TYPE_INT) {
        printf("%d", val->as.integer);
    } else {
        printf("nil");
    }
}

void value_copy(struct value *left, struct value *right) {
    memcpy(left, right, sizeof(struct value));
}

// arith
#define arith_op(name, op) \
void value_ ## name (struct value *result, const struct value *left, const struct value *right) { \
    assert(left->type == right->type); \
    if(left->type == TYPE_INT) { \
        value_int(result, left->as.integer op right->as.integer); \
    } else if(left->type == TYPE_FLOAT) { \
        value_int(result, left->as.floatp op right->as.floatp); \
    } \
}
arith_op(add, +)
arith_op(sub, -)
arith_op(mul, *)
arith_op(div, /)
void value_mod(struct value *result, const struct value *left, const struct value *right) {
    assert(left->type == right->type);
    if(left->type == TYPE_INT) {
        value_int(result, left->as.integer % right->as.integer);
    }
}
