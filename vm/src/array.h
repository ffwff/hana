#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include <stdlib.h>
#include <string.h>

#define array(type)      \
    struct {             \
        type* data;      \
        size_t length;   \
        size_t capacity; \
    }

#define array_init(type)  \
    {                 \
        .data = calloc(1, sizeof(type)), \
        .length = 0,  \
        .capacity = 1 \
    }

#define array_free(array)   \
    do {                    \
        free(array.data);   \
    } while (0)

#ifdef __cplusplus
#define VOID_CAST(data,x) (decltype(data))(x)
#else
#define VOID_CAST(data,x) x
#endif
#define array_push(array, element)                   \
    do {                                             \
        if(array.length == array.capacity) { \
            array.capacity *= 2; \
            array.data = VOID_CAST(array.data, realloc(array.data, sizeof(*array.data)*array.capacity)); \
        } \
        array.data[array.length] = element; \
        array.length++; \
    } while (0)

#define array_append(array, src) \
do { \
    if(array.length+src.length > array.capacity) { \
        array.capacity = array.length+src.length; \
        array.data = VOID_CAST(array.data, realloc(array.data, sizeof(*array.data)*array.capacity)); \
    } \
    memcpy(array.data[array.length], src.data, sizeof(*src.data)*src.length); \
} while (0)

#define array_pop(array) \
    do {                 \
        array.length--;  \
    } while(0)

#define array_top(array) array.data[array.length-1]

#ifdef __cplusplus
}
#endif
