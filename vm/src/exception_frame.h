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
    uint32_t unwind_stack_size; // stack size target
    uint32_t end_ip; // ip to end of try block
};
void exception_frame_init(struct exception_frame *frame, struct exception_frame *prev,
                          uint32_t unwind_stack_size, uint32_t end_ip);
void exception_frame_free(struct exception_frame *frame);

#ifdef __cplusplus
}
#endif
