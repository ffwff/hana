#pragma once
#include <stdint.h>
#include <stdbool.h>
#include "dict.h"
#include "array_obj.h"

struct vm;
struct hmap;
struct array_obj;
struct string;

#define TYPE_NIL 0
#define TYPE_INT 1
#define TYPE_FLOAT 2
#define TYPE_NATIVE_FN 3
#define TYPE_FN 4
#define TYPE_STR 5
#define TYPE_DICT 6
#define TYPE_ARRAY 7
#define TYPE_INTERPRETER_ERROR 127
#define TYPE_INTERPRETER_ITERATOR 128

typedef void (*value_fn)(struct vm *vm, uint16_t nargs);
struct __attribute__((packed)) value {
    union {
        int64_t integer;
        double floatp;
        void *ptr;
    } as;
    uint8_t type;
};

struct env;

struct value value_int(int64_t);
struct value value_float(double);
struct value value_str(const char*, const struct vm*);
struct value value_function(uint32_t ip, uint16_t nargs, struct env *env, const struct vm *);
struct value value_dict(const struct vm *);
struct value value_dict_n(size_t n, const struct vm *);
struct value value_array(const struct vm *);
struct value value_array_n(size_t n, const struct vm *);
struct value value_interpreter_error();

struct value value_add(const struct value left, const struct value right, const struct vm *);
struct value value_sub(const struct value left, const struct value right, const struct vm *);
struct value value_mul(const struct value left, const struct value right, const struct vm *);
struct value value_div(const struct value left, const struct value right, const struct vm *);
struct value value_mod(const struct value left, const struct value right, const struct vm *);

struct value value_bitwise_and(const struct value left, const struct value right, const struct vm *);
struct value value_bitwise_or (const struct value left, const struct value right, const struct vm *);
struct value value_bitwise_xor(const struct value left, const struct value right, const struct vm *);

int value_iadd(struct value left, const struct value right);
int value_imul(struct value left, const struct value right);

struct value value_lt (const struct value left, const struct value right, const struct vm*);
struct value value_leq(const struct value left, const struct value right, const struct vm*);
struct value value_gt (const struct value left, const struct value right, const struct vm*);
struct value value_geq(const struct value left, const struct value right, const struct vm*);
struct value value_eq (const struct value left, const struct value right, const struct vm*);
struct value value_neq(const struct value left, const struct value right, const struct vm*);

bool value_is_true(const struct value);
struct dict *value_get_prototype(const struct vm *vm, const struct value val);

// TODO move this somewhere else
#ifdef DEBUG
#define debug_assert(x) if(!(x)){ *((void*)0); }
#else
#define debug_assert(x)
#endif
static inline struct value value_pointer(uint8_t tag, void *ptr) {
    return (struct value){
        .type = tag,
        .as.ptr = ptr,
    };
}
// TODO: remove redundant checks from opcodes
static inline void *value_get_pointer(struct value val) {
    return val.as.ptr;
}
static inline int64_t value_get_int(struct value val) {
    return val.as.integer;
}
static inline double value_get_float(struct value val) {
    return val.as.floatp;
}
#undef debug_assert