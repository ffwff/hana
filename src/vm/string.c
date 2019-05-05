#include "string_.h"

void string_init(struct string_header *string, const char *s) {
    string->length = strlen(s);
    char *data = (char*)((char*)string+sizeof(struct string_header));
    strcpy(data, s);
}

struct string_header *string_alloc(size_t n) {
    struct string_header *s = (struct string_header *)malloc(sizeof(struct string_header) + n + 1);
    s->length = n;
    return s;
}