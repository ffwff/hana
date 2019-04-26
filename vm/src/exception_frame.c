#include "exception_frame.h"

void exception_frame_init(struct exception_frame *frame,
                          struct exception_frame *prev,
                          uint32_t unwind_stack_size,
                          uint32_t end_ip) {
    frame->prev = prev;
    frame->handlers = (a_exception_frame_data)array_init(struct exception_frame_data);
    frame->unwind_stack_size = unwind_stack_size;
    frame->end_ip = end_ip;
}

void exception_frame_free(struct exception_frame *frame) {
    for(size_t i = 0; frame->handlers.length; i++)
        value_free(&frame->handlers.data[i].etype);
    array_free(frame->handlers);
}
