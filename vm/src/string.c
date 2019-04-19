#include "string_.h"

void string_init(struct string_header *string, const char *s) {
    string->refs = 1;
    string->length = strlen(s);
    char *data = (char*)((char*)string+sizeof(struct string_header));
    strcpy(data, s);
}

void string_free(struct string_header *string) {
    string->refs--;
}
