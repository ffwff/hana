#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "vm.h"
#include "value.h"

struct dict;
struct function;

void eframe_set_handler(struct eframe *frame, const struct dict *proto, const struct function *fn);

#ifdef __cplusplus
}
#endif
