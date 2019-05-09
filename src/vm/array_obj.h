#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include "array.h"
#include "value.h"

typedef array(struct value) array_obj;
array_obj *array_obj_malloc(void);
array_obj *array_obj_malloc_n(size_t n);

#ifdef __cplusplus
}
#endif
