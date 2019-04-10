#include <stdio.h>
#include <assert.h>
#include "vm.h"

int main() {
    struct vm m;
    vm_init(&m);

    // uint8
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 10);

    array_push(m.code, OP_PUSH32);
    vm_code_push32(&m, 65538);

    array_push(m.code, OP_ADD);

    vm_execute(&m);

    vm_free(&m);
    return 0;
}
