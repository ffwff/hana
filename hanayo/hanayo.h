#pragma once

#define fn(name) void name(struct vm *vm, int nargs)
#include <string>
#include "vm/src/vm.h"

namespace hanayo {

fn(print);
fn(input);

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
}

namespace integer {
    fn(constructor);
}
namespace float_ {
    fn(constructor);
    fn(round);
}

namespace record {
    fn(constructor);
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
}

std::string _to_string(struct value &val);
void _init(struct vm *m);

}

#undef fn
