#pragma once
#include <stdlib.h>
#include <string.h>

void *rcalloc(size_t nelems, size_t size);
void *rrealloc(void *ptr, size_t nelems, size_t size, size_t new_size);

#define array(type)      \
    struct {             \
        type* data;      \
        size_t length;   \
        size_t capacity; \
    }

#define array_init(type)                  \
    {                                     \
        .data = rcalloc(1, sizeof(type)), \
        .length = 0,                      \
        .capacity = 1                     \
    }
#define array_init_n(type, n)             \
    {                                     \
        .data = rcalloc(n, sizeof(type)), \
        .length = n,                      \
        .capacity = n                     \
    }

#define array_free(array)   \
    do {                    \
        free(array.data);   \
    } while (0)

#define array_push(array, element)                                 \
    do {                                                           \
        if (array.length == array.capacity) {                      \
            array.data = rrealloc(                                 \
                array.data, array.capacity, sizeof(array.data[0]), \
                sizeof(*array.data) * array.capacity * 2);         \
            array.capacity *= 2;                                   \
        }                                                          \
        array.data[array.length] = element;                        \
        array.length++;                                            \
    } while (0)

#define array_pop(array) \
    do {                 \
        array.length--;  \
    } while(0)

#define array_top(array) array.data[array.length-1]