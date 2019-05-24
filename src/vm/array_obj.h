#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include "array.h"

typedef array(struct value) array_obj;
struct vm;
array_obj *array_obj_malloc(const struct vm *vm);
array_obj *array_obj_malloc_n(size_t n, const struct vm *vm);
array_obj *array_obj_repeat(array_obj *array, size_t times, const struct vm *vm);

#ifdef __cplusplus
}
#endif
