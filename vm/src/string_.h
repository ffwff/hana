#pragma once
#include "array.h"
#include <stdint.h>

struct string_header { // string = [header][data][0]
    uint32_t refs;
    uint32_t length;
};

void string_init(struct string_header *, const char *);
#define string_size(data) (sizeof(struct string_header)+strlen(data)+1)
struct string_header *string_alloc(size_t n);
#define string_data(string) (char*)((char*)string+sizeof(struct string_header))
#define string_len(string) (string->length)
#define string_at(string, i) *((char*)((char*)string+sizeof(struct string_header)+i))
#define string_cmp(s1, s2) strcmp(string_data(s1), string_data(s2))
