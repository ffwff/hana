#include <stdio.h>
#include <assert.h>
#include "vm.h"

int main() {
    struct vm m;
    vm_init(&m);

    // uint8
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 10);

    array_push(m.code, OP_PUSH16);
    vm_code_push16(&m, (uint16_t)256);

    array_push(m.code, OP_ADD);

    vm_execute(&m);

    vm_free(&m);
    return 0;
}
