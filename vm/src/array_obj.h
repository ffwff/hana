#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "array.h"
#include "value.h"
typedef array(struct value) array_obj_data;
struct array_obj {
    uint32_t refs;
    array_obj_data data;
};
void array_obj_init(struct array_obj *);
void array_obj_init_n(struct array_obj *, size_t n);
void array_obj_free(struct array_obj *);

#ifdef __cplusplus
}
#endif
