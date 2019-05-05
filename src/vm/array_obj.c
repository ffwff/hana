#include "array_obj.h"

void array_obj_init(struct array_obj *array_obj) {
    array_obj->data = (array_obj_data)array_init(struct value);
}

void array_obj_init_n(struct array_obj *array_obj, size_t n) {
    array_obj->data = (array_obj_data)array_init_n(struct value, n);
}

void array_obj_free(struct array_obj *array_obj, __attribute__((unused)) void *p)
{
    array_free(array_obj->data);
}
