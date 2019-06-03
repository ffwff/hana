#pragma once
#include <stdint.h>
#include <stdbool.h>
#include "dict.h"
#include "array_obj.h"

struct vm;
struct hmap;
struct array_obj;
struct string;

#define RESERVED_NAN 0x7ff
#define TYPE_FLOAT 0
#define TYPE_INT 1
#define TYPE_NATIVE_FN 2
#define TYPE_FN 3
#define TYPE_STR 4
#define TYPE_DICT 5
#define TYPE_ARRAY 6
#define TYPE_INTERPRETER_ERROR 7
#define TYPE_INTERPRETER_ITERATOR 8
#define TYPE_NIL 9

/*
    1111111111110000
    01111111 11111010 00000000 00000000 00000000 00000000 00000000 00000000
    seeeeeee|eeeemmmm|mmmmmmmm|mmmmmmmm|mmmmmmmm|mmmmmmmm|mmmmmmmm|mmmmmmmm
    ^            ^^^^ tagging bits (T)
    ^            ^ initial mantissa bit (whether it's a signalling/quiet NaN)
    ^ signed bit (ignored)

    we'll assume the architecture is using IEEE 754 double precision floating
    point, and the OS allocates userland memory in the lower half of memory.
    NaN values inside doubles allow us to store a 52-bit payload inside the mantissa
    we can store 48-bit pointers/32-bit integers in the lower 48-bit region
    and 4 additional higher bits for tagging

    possible values for T:
    - int
    - native_fn
    - fn
    - str
    - dict
    - array
    - [interpreter data]

*/

struct __attribute__((packed)) value {
    union {
        double floatp;
        uint64_t bin;
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
        struct __attribute__((packed)) {
            uint64_t payload : 48;
            uint8_t tag_bits : 4;        // must be larger than 0
            uint16_t reserved_nan : 12;  // must be positive NaN (0x7ff0)
        } bits;
        struct __attribute__((packed)) {
            uint32_t lower32;
            uint32_t upper32;
        };
#else
#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
        struct __attribute__((packed)) {
            uint16_t reserved_nan : 12;  // must be positive NaN (0x7ff0)
            uint8_t tag_bits : 4;        // must be larger than 0
            uint64_t payload : 48;
        } bits;
        struct __attribute__((packed)) {
            uint32_t upper32;
            uint32_t lower32;
        };
#else
#error "does not support this system"
#endif
#endif
    } as;
};

typedef void (*value_fn)(struct vm *vm, uint16_t nargs);

struct env;

struct value value_int(int32_t);
struct value value_float(double);
struct value value_str(const char*, const struct vm*);
struct value value_function(uint32_t ip, uint16_t nargs, struct env *env, const struct vm *);
struct value value_dict(const struct vm *);
struct value value_dict_n(size_t n, const struct vm *);
struct value value_array(const struct vm *);
struct value value_array_n(size_t n, const struct vm *);
int value_get_type(struct value val);

void value_print(struct value);

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

#define assert(x) if(!(x)){ *((void*)0); }
static inline struct value value_pointer(uint8_t tag, void *ptr) {
    uint64_t low_bits = (uint64_t)ptr & 0xffffffffffff;
    assert(low_bits == (uint64_t)ptr);
    assert(tag < 16 && tag > 0);  // we can only store 4 bits
    return (struct value){
        .as.bits.reserved_nan = RESERVED_NAN,
        .as.bits.tag_bits = tag,
        .as.bits.payload = low_bits};
}
static inline uint16_t value_get_tag(struct value val) {
    assert(val.as.bits.reserved_nan == RESERVED_NAN && val.as.bits.tag_bits > 0);
    return val.as.bits.tag_bits;
}
// TODO: remove redundant checks from opcodes
static inline void *value_get_pointer(uint8_t tag, struct value val) {
    assert(tag != TYPE_INT);
    assert(val.as.bits.reserved_nan == RESERVED_NAN && val.as.bits.tag_bits == tag);
    return (void *)val.as.bits.payload;
}
static inline int32_t value_get_int(struct value val) {
    assert(val.as.bits.reserved_nan == RESERVED_NAN && val.as.bits.tag_bits == TYPE_INT);
    return (int32_t)val.as.lower32;
}
static inline void value_set_int(struct value *val, int32_t n) {
    val->as.bits.reserved_nan = RESERVED_NAN;
    val->as.bits.tag_bits = TYPE_INT;
    val->as.lower32 = n;
}
static inline double value_get_float(struct value val) {
    assert(!isnan(val.as.floatp));
    return val.as.floatp;
}
static inline void value_set_float(struct value *val, double n) {
    val->as.floatp = n;
    assert(val->as.bits.reserved_nan != RESERVED_NAN);
}
#undef assert