#include <stdio.h>
#include <assert.h>
#include "vm.h"
#include "map.h"

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

#ifdef SECTION_MAP
    // map
    struct map map;
    map_init(&map);
    struct value v = {
        .type = TYPE_INT,
        .as.integer = 0
    };
    v.as.integer = 13;
    map_set(&map, "A", &v);
    v.as.integer = 33;
    map_set(&map, "B", &v);
    v.as.integer = 37;
    map_set(&map, "C", &v);
    map_print(&map);
    printf("\n");

    v.as.integer = 1337;
    map_set(&map, "C", &v);
    map_print(&map);
    printf("\n");

    map_del(&map, "B");
    map_print(&map);

    map_free(&map);
#endif

#ifdef SECTION_VARIABLE
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 100);

    array_push(m.code, OP_SET);
    vm_code_pushstr(&m, "test"); // => []

    array_push(m.code, OP_POP);

    array_push(m.code, OP_GET);
    vm_code_pushstr(&m, "test"); // => [100]
#endif

#ifdef SECTION_JMP // TODO
    array_push(m.code, OP_JMP);
    array_push(m.code, 4);

    array_push(m.code, OP_PUSH8);
    array_push(m.code, 13);

    array_push(m.code, OP_PUSH8); // 4
    array_push(m.code, 33);

    array_push(m.code, OP_JCOND);
    array_push(m.code, 10);

    array_push(m.code, OP_PUSH8);
    array_push(m.code, 33);

    array_push(m.code, OP_PUSH8); // 10
    array_push(m.code, 37);
#endif

#ifdef SECTION_SCOPED
    array_push(m.code, OP_ENV_INHERIT);
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 42);
    array_push(m.code, OP_SET);
    vm_code_pushstr(&m, "a");
    array_push(m.code, OP_POP);

    array_push(m.code, OP_ENV_INHERIT);
    array_push(m.code, OP_PUSH8);
    array_push(m.code, 50);
    array_push(m.code, OP_SET);
    vm_code_pushstr(&m, "b");
    array_push(m.code, OP_POP);
    array_push(m.code, OP_GET);
    vm_code_pushstr(&m, "a"); // => 42
    array_push(m.code, OP_POP);
    array_push(m.code, OP_ENV_POP);

    array_push(m.code, OP_ENV_POP);

#endif

#ifdef SECTION_DICT_RC_SIMPLE
    // should free everything
    array_push(m.code, OP_DICT_NEW);
    array_push(m.code, OP_POP);
#endif

#define SECTION_DICT_RC_CMPLX
#ifdef SECTION_DICT_RC_CMPLX
    // should free everything
    array_push(m.code, OP_DICT_NEW);
    array_push(m.code, OP_SET);
    vm_code_pushstr(&m, "a");
    array_push(m.code, OP_POP);

    array_push(m.code, OP_GET);
    vm_code_pushstr(&m, "a");
    array_push(m.code, OP_SET);
    vm_code_pushstr(&m, "b");
    array_push(m.code, OP_POP);
#endif

    array_push(m.code, OP_HALT);
    vm_execute(&m);

    vm_free(&m);
    return 0;
}
