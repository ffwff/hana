#include "fastalloc.h"
#include <stdio.h>
#include <string.h>
#include <assert.h>

// Bits
#define count_bits __builtin_popcountll
#define first_set_bit __builtin_ffsl
#define round8(x) ((x)&~7)

/* a=target variable, b=bit number to act upon 0-n */
#define BIT_SET(a,b) ((a) |= (1ULL<<(b)))
#define BIT_CLEAR(a,b) ((a) &= ~(1ULL<<(b)))
#define BIT_FLIP(a,b) ((a) ^= (1ULL<<(b)))
#define BIT_CHECK(a,b) (!!((a) & (1ULL<<(b))))

// fast alloc impl
void fa_init() {
    memset(chunk, 0, FA_CHUNKSIZE);
    chunk_free = 0b1111111111111111111111111111111111111111111111111111011001100101;//UINT64_MAX;
}

void *fa_malloc(uint8_t sz) {
    sz = round8(sz)/8;
    for(uint8_t first_bit = first_set_bit(chunk_free)-1;
        first_bit < FA_MAXELEMENTS-sz;
        first_bit = first_set_bit(chunk_free << first_bit)) {
        for(uint8_t i = 0; i < sz; i++)
            if(!BIT_CHECK(chunk_free, first_bit+i))
                goto next;
        for(uint8_t i = 0; i < sz; i++)
            BIT_CLEAR(chunk_free, first_bit+i);
        printf("%d\n", first_bit);
        return &chunk[first_bit];
    next:;
    }
    return NULL;
}

void fa_free(void *ptr, uint8_t sz) {
    sz = round8(sz)/8;
    uint8_t bit = ((intptr_t)ptr-(intptr_t)chunk);
    for(uint8_t i = bit; i < bit+sz; i++)
        BIT_SET(chunk_free, i);
}
