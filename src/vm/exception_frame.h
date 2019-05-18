#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "vm.h"
#include "value.h"

struct dict;
struct function;
struct exframe;

void exframe_set_handler(struct exframe *frame, const struct dict *proto, const struct function *fn);
size_t exframe_native_stack_depth(const struct exframe *);

#ifdef __cplusplus
}
#endif
