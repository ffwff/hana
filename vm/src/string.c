#include "string_.h"

void string_init(struct string *string, const char *s) {
    string->refs = 1;
    string->data = (a_char)array_init_n(char, strlen(s)+1);
    memcpy(string->data.data, s, strlen(s));
    string->data.data[strlen(s)] = 0;
}

void string_free(struct string *string) {
    string->refs--;
    if(string->refs == 0)
        array_free(string->data);
}
