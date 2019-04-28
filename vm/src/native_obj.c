#include "native_obj.h"

void native_obj_init(struct native_obj *obj, void *data, native_obj_free_fn free) {
    obj->refs = 1;
    obj->data = data;
    obj->free = free;
}
void native_obj_free(struct native_obj *obj) {
    if(obj->refs == 0)
        obj->free(obj->data);
}
