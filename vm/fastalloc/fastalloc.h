#pragma once

#include <stddef.h>
#include <stdint.h>

#define FA_BLOCKSIZE 8
#define FA_MAXELEMENTS 64
#define FA_CHUNKSIZE (FA_BLOCKSIZE*FA_MAXELEMENTS)

uint8_t chunk[FA_CHUNKSIZE];
uint64_t chunk_free; // 0 => allocated, 1 => free
// bits (bytes?) are in reverse when printed

void fa_init();
void *fa_malloc(uint8_t sz);
void fa_free(void *ptr, uint8_t sz);
