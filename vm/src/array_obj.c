#include "array_obj.h"

void array_obj_init(struct array_obj *array_obj) {
    array_obj->refs = 1;
    array_obj->data = (array_obj_data)array_init(struct value);
}

void array_obj_init_n(struct array_obj *array_obj, size_t n) {
    array_obj->refs = 1;
    array_obj->data = (array_obj_data)array_init_n(struct value, n);
}

void array_obj_free(struct array_obj *array_obj) {
    //for(size_t i = 0; i < array_obj->data.length; i++)
        //value_free(&array_obj->data.data[i]);
    array_free(array_obj->data);
}
