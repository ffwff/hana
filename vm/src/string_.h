#pragma once
#include "array.h"

typedef array(char) a_char;
struct string {
    a_char data;
    size_t refs;
};

void string_init(struct string *, const char *);
void string_free(struct string *);
#define string_data(string) (string->data.data)
#define string_len(string) (string->data.length)
#define string_at(string, i) (string->data.data[i])
#define string_cmp(s1, s2) strcmp(string_data(s1), string_data(s2))
