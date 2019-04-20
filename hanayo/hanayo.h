#pragma once

#define fn(name) void name(struct vm *vm, int nargs)
#include <string>
#include <cassert>
#include "vm/src/vm.h"

namespace hanayo {

// io
fn(print);
fn(input);
fn(fopen);
fn(fread);
fn(fwrite);

namespace string {
    fn(constructor);
    // methods
    fn(bytesize);
    fn(length);
    fn(delete_);
    fn(copy);
    fn(at);
    fn(index);
    fn(insert);
    fn(split);
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

std::string _to_string(struct value &val);
void _init(struct vm *m);

struct value _arg(struct vm *vm, value::value_type type);

}

#undef fn
