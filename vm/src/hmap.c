#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "xxhash.h"
#include "hmap.h"

#define INITIAL_SIZE 2
#define LOAD_FACTOR 0.7

// hashing functions
static size_t hmap_index(struct hmap *hmap, const char *key) {
    const uint32_t hash = XXH32(key, strlen(key), 0);
    return hash & (hmap->data.length-1);
}

// hashmap
void hmap_init(struct hmap *hmap) {
    hmap->occupied = 0;
    hmap->data = (a_hmap_buckets)array_init_n(a_hmap_entry, INITIAL_SIZE);
    for(size_t i = 0; i < INITIAL_SIZE; i++)
        hmap->data.data[i] = (a_hmap_entry)array_init(a_hmap_entry);
}

void hmap_free(struct hmap *hmap) {
    for(size_t i = 0; i < hmap->data.length; i++) {
        for(size_t j = 0; j < hmap->data.data[i].length; j++) {
            free(hmap->data.data[i].data[j].key);
            value_free(&hmap->data.data[i].data[j].val);
        }
        array_free(hmap->data.data[i]);
    }
    array_free(hmap->data);
    hmap->occupied = 0;
}

struct value *hmap_get(struct hmap *hmap, const char *key) {
    const a_hmap_entry entry = hmap->data.data[hmap_index(hmap, key)];
    //printf("LEN: %d\n", entry.length);
    for(size_t i = 0; i < entry.length; i++)
        if(strcmp(entry.data[i].key, key) == 0)
            return &entry.data[i].val;
    return NULL;
}
struct value *hmap_get_hash(struct hmap *hmap, const char *key, const uint32_t hash) {
    // NOTE: compilers can optimize modulos between 2 32-bit numbers (at least in x86)
    const a_hmap_entry entry = hmap->data.data[(uint64_t)hash & (hmap->data.length-1)];
    for(size_t i = 0; i < entry.length; i++)
        if(strcmp(entry.data[i].key, key) == 0)
            return &entry.data[i].val;
    return NULL;
}

struct value *_hmap_set(struct hmap *hmap, const char *key, struct value *val, int noalloc, int *has_grown) {
//     printf("SET %s\n", key);
    if(has_grown != NULL) *has_grown = 0;
    if(!noalloc) {
        // expand if load factor > lf threshold
        const float load = ((float)hmap->occupied)/((float)hmap->data.length);
        //printf("load factor: %f\n", load);
        if(load > LOAD_FACTOR) {
            struct hmap tmp;
            tmp.occupied = 0;
            tmp.data = (a_hmap_buckets)array_init_n(a_hmap_entry, 2*hmap->data.length);
            for(size_t i = 0; i < tmp.data.length; i++)
                tmp.data.data[i] = (a_hmap_entry)array_init(a_hmap_entry);
            for(size_t i = 0; i < hmap->data.length; i++)
                for(size_t j = 0; j < hmap->data.data[i].length; j++)
                    _hmap_set(&tmp, hmap->data.data[i].data[j].key,
                            &hmap->data.data[i].data[j].val, 1, 0);
            for(size_t i = 0; i < hmap->data.length; i++)
                array_free(hmap->data.data[i]);
            array_free(hmap->data);
            hmap->occupied = tmp.occupied;
            hmap->data = tmp.data;
            if(has_grown != NULL) *has_grown = 1;
        }
    }
    // set
    a_hmap_entry *entry = &hmap->data.data[hmap_index(hmap, key)];
    for(size_t i = 0; i < entry->length; i++) {
        if(strcmp(entry->data[i].key, key) == 0) {
            if(noalloc) {
                entry->data[i].val.type = val->type;
                entry->data[i].val.as = val->as;
            } else {
                value_free(&entry->data[i].val);
                value_copy(&entry->data[i].val, val);
            }
            return &entry->data[i].val;
        }
    }
    struct hmap_entry mentry = {
        .key = noalloc ? (char*)key : strdup(key)
    };
    if(noalloc) {
        mentry.val.type = val->type;
        mentry.val.as = val->as;
    } else {
        value_copy(&mentry.val, val);
    }
    array_ptr_push(entry, mentry);
    if(entry->length == 1) hmap->occupied++;
    return &entry->data[entry->length-1].val;
}

void hmap_del(struct hmap *hmap, const char *key) {
    a_hmap_entry *entry = &hmap->data.data[hmap_index(hmap, key)];
    if(entry->length == 1) {
        free(entry->data[0].key);
        value_free(&entry->data[0].val);
        entry->length--;
        if(entry->length == 0) hmap->occupied--;
    } else {
        for(size_t i = 0; i < entry->length; i++)
            if(strcmp(entry->data[i].key, key) == 0) {
                // free entry
                free(entry->data[i].key);
                value_free(&entry->data[i].val);
                // move it
                i++;
                for(; i < entry->length; i++)
                    entry->data[i-1] = entry->data[i];
                entry->length--;
                if(entry->length == 0) hmap->occupied--;
                return;
            }
    }
}

void hmap_print(struct hmap *hmap) {
    printf("buckets: %ld/%ld\n", hmap->occupied, hmap->data.length);
    for(size_t i = 0; i < hmap->data.length; i++) {
        for(size_t j = 0; j < hmap->data.data[i].length; j++) {
            printf("%s: ", hmap->data.data[i].data[j].key);
            value_print(&hmap->data.data[i].data[j].val);
            printf("\n");
        }
    }
}
