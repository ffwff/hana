#pragma once
#include "array.h"

struct string_header { // string = [header][data][0]
    size_t refs, length;
};

void string_init(struct string_header *, const char *);
#define string_size(data) (sizeof(struct string_header)+strlen(data)+1)
void string_free(struct string_header *);
#define string_data(string) (char*)((char*)string+sizeof(struct string_header))
#define string_len(string) (string->length)
#define string_at(string, i) *((char*)(string+sizeof(struct string_header)+i))
#define string_cmp(s1, s2) strcmp(string_data(s1), string_data(s2))
