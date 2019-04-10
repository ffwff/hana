#include <stdio.h>
#include <assert.h>
#include "vm.h"

int main() {
    struct vm m;
    vm_init(&m);

#ifdef SECTION_INTP
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 10);

    array_push(m.code, OP_PUSH32);
    vm_code_push32(&m, 65538);

    array_push(m.code, OP_ADD); // => 65548
#endif

#define SECTION_STRINGP
#ifdef SECTION_STRINGP
    // append
    array_push(m.code, OP_PUSHSTR);
    vm_code_pushstr(&m, "Hello");

    array_push(m.code, OP_PUSHSTR);
    vm_code_pushstr(&m, " World!");

    array_push(m.code, OP_ADD); // => Hello World!

    // repeat
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 2);
    array_push(m.code, OP_MUL); // => Hello World!Hello World!
#endif

    array_push(m.code, OP_HALT);
    vm_execute(&m);

    vm_free(&m);
    return 0;
}
