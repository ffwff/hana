#include <stddef.h>
#include <stdio.h>
#include "fastalloc.h"
#include "../src/value.h"

int main() {
    fa_init();
    void *f;
    f = fa_malloc(16);
    printf("0x%lx / %lx \n", f, chunk_free);
    f = fa_malloc(16);
    printf("0x%lx / %lx \n", f, chunk_free);
    fa_free(f, 16);
    printf("%lx \n", chunk_free);
    return 0;
}
