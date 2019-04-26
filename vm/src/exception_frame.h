#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include "vm.h"
#include "value.h"
struct exception_frame_data {
    struct value etype, fn;
};
typedef array(struct exception_frame_data) a_exception_frame_data;

struct exception_frame {
    struct exception_frame *prev;
    a_exception_frame_data handlers;
    struct env *unwind_env; // localenv rewind target
    uint32_t unwind_stack_size; // stack size rewind target
};
void exception_frame_init(struct exception_frame *frame, struct exception_frame *prev);
void exception_frame_init_vm(struct exception_frame *frame, struct vm *vm);
void exception_frame_free(struct exception_frame *frame);
struct value *exception_frame_get_handler_for_error(struct exception_frame *frame, const struct vm *vm, const struct value *error);
void exception_frame_unwind(struct exception_frame *frame, struct vm *vm);

#ifdef __cplusplus
}
#endif
