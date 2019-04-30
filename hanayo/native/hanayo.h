#pragma once

#define fn(name) void name(struct vm *vm, int nargs)
#include <string.h>
#include <assert.h>
#include "vm/src/vm.h"

namespace hanayo {

// Helper functions
struct Value { // wrapper
    struct value v;
    Value() { v.type = value::value_type::TYPE_NIL; }
    ~Value() { value_free(&v); }
    struct value deref() {
        struct value v = this->v;
        this->v.type = value::value_type::TYPE_NIL;
        return v;
    }
    // move/copy
    static Value move(struct value &val) {
        Value v;
        v.v = val;
        val.type = value::value_type::TYPE_NIL;
        val.as.ptr = 0;
        return v;
    }
    static void move(Value &dst, Value &src) {
        value_free(&dst.v);
        dst.v = src.v;
        src.v.type = value::value_type::TYPE_NIL;
        src.v.as.ptr = 0;
    }
    static Value copy(struct value &val) {
        Value v;
        value_copy(&v.v, &val);
        return v;
    }
    // disable operators
    Value &operator=(Value &other) = delete;
    // deref
    operator value&() { return v; }
    operator value*() { return &v; }
};

char *_to_string(struct value &val);
void _init(struct vm *m);
Value _top(struct vm *vm);
Value _pop(struct vm *vm);
void _push(struct vm *vm, Value &val);
Value _arg(struct vm *vm, value::value_type type);

// io
fn(print);
fn(input);
fn(eval);
fn(exit);

// ffi
namespace ffi {
    fn(function);
    fn(call);
    enum type {
        UInt8, Int8, UInt16, Int16, UInt32, Int32, UInt64, Int64,
        Float32, Float64, UChar, Char, UShort, Short, ULong, Long,
        Pointer, String, Void
    };
    namespace rcpointer {
        extern struct value prototype;
        fn(constructor);
    };
};

namespace string {
    fn(constructor);
    fn(reserve);
    // methods
    fn(bytesize);
    fn(length);
    fn(delete_);
    fn(copy);
    fn(at);
    fn(index);
    fn(insert);
    fn(split);
    fn(startswith);
    fn(endswith);
    fn(shrink_);
}

namespace integer {
    fn(constructor);
}
namespace float_ {
    fn(constructor);
    fn(round);
}

namespace array {
    fn(constructor);
    // methods
    fn(length);
    fn(delete_);
    fn(copy);
    fn(at);
    fn(index);
    fn(insert);
    fn(push);
    fn(pop);
    fn(sort);
    fn(sort_); // sort!
    fn(map);
    fn(filter);
    fn(reduce);
    fn(join);
}

namespace record {
    fn(constructor);
    fn(keys);
    fn(is_record);
    fn(copy);
}

}

#undef fn
