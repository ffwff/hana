#include <stdio.h>
#include <limits.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

fn(exit) {
    assert(nargs == 1);
    auto code = _arg(vm, value::TYPE_INT);
    ::exit(code.as.integer);
}
