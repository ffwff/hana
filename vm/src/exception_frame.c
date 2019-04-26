#include "exception_frame.h"
#include "value.h"

void exception_frame_init(struct exception_frame *frame, struct exception_frame *prev) {
    frame->prev = prev;
    frame->handlers = (a_exception_frame_data)array_init(struct exception_frame_data);
}

void exception_frame_init_vm(struct exception_frame *frame, struct vm *vm) {
    frame->unwind_env = vm->localenv;
    frame->unwind_stack_size = vm->stack.length;
}

void exception_frame_free(struct exception_frame *frame) {
    for(size_t i = 0; i < frame->handlers.length; i++)
        value_free(&frame->handlers.data[i].etype);
    array_free(frame->handlers);
}

struct value *exception_frame_get_handler_for_error(struct exception_frame *frame, const struct vm *vm, const struct value *error) {
    for(size_t i = 0; i < frame->handlers.length; i++) {
        if(value_get_prototype(vm, error) == frame->handlers.data[i].etype.as.dict) {
            return &frame->handlers.data[i].fn;
        }
    }
    return NULL;
}

void exception_frame_unwind(struct exception_frame *frame, struct vm *vm) {
    // unwind stack
    //printf("UNWIND! %d\n", frame->unwind_stack_size);
    for(size_t i = vm->stack.length; i > frame->unwind_stack_size; i--)
        value_free(&vm->stack.data[i]);
    vm->stack.length = frame->unwind_stack_size;
    // env
    while(vm->localenv != frame->unwind_env) {
        struct env *localenv = vm->localenv->parent;
        env_free(vm->localenv);
        free(vm->localenv);
        vm->localenv = localenv;
    }
}
