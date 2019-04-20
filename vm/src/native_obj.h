#pragma once
#include <stddef.h>

typedef void (*native_obj_free_fn)(void *data);
struct native_obj {
    size_t refs;
    void *data;
    native_obj_free_fn free;
};

void native_obj_init(struct native_obj*, void *data, native_obj_free_fn free);
void native_obj_free(struct native_obj*);
