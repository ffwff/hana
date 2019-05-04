#include "native_obj.h"

void native_obj_init(struct native_obj *obj, void *data, native_obj_free_fn free) {
    obj->data = data;
    obj->free = free;
}
void native_obj_free(struct native_obj *obj, __attribute__((unused)) void *p)
{
    obj->free(obj->data);
}
