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
    hmap->keys = (a_hmap_keys)array_init(char*);
}

void hmap_free(struct hmap *hmap) {
    for(size_t i = 0; i < hmap->data.length; i++) {
        for(size_t j = 0; j < hmap->data.data[i].length; j++) {
            free(hmap->data.data[i].data[j].key);
            //value_free(&hmap->data.data[i].data[j].val);
        }
        array_free(hmap->data.data[i]);
    }
    array_free(hmap->data);
    // NOTE: strings in hmap->keys are owned by hmap->data's entries
    // no need to free it
    array_free(hmap->keys);
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
    const a_hmap_entry entry = hmap->data.data[(uint64_t)hash & (hmap->data.length-1)];
    for(size_t i = 0; i < entry.length; i++)
        if(strcmp(entry.data[i].key, key) == 0)
            return &entry.data[i].val;
    return NULL;
}

struct value *_hmap_set(struct hmap *hmap, const char *key, struct value *val, int noalloc) {
    if(!noalloc) {
        // expand if load factor > lf threshold
        const float load = ((float)hmap->occupied)/((float)hmap->data.length);
        if(load > LOAD_FACTOR) {
            struct hmap tmp;
            tmp.occupied = 0;
            tmp.data = (a_hmap_buckets)array_init_n(a_hmap_entry, 2*hmap->data.length);
            tmp.keys = hmap->keys; // move keys
            for(size_t i = 0; i < tmp.data.length; i++)
                tmp.data.data[i] = (a_hmap_entry)array_init(a_hmap_entry);
            for(size_t i = 0; i < hmap->data.length; i++)
                for(size_t j = 0; j < hmap->data.data[i].length; j++)
                    _hmap_set(&tmp, hmap->data.data[i].data[j].key,
                            &hmap->data.data[i].data[j].val, 1);
            for(size_t i = 0; i < hmap->data.length; i++)
                array_free(hmap->data.data[i]);
            array_free(hmap->data);
            hmap->occupied = tmp.occupied;
            hmap->data = tmp.data;
            hmap->keys = tmp.keys; // give it back
        }
    }
    // set
    a_hmap_entry *entry = &hmap->data.data[hmap_index(hmap, key)];
    for(size_t i = 0; i < entry->length; i++) {
        if(strcmp(entry->data[i].key, key) == 0) {
            if(noalloc) {
                // we don't need to copy if we're just resizing it
                // just move it
                entry->data[i].val.type = val->type;
                entry->data[i].val.as = val->as;
            } else {
                //value_free(&entry->data[i].val);
                value_copy(&entry->data[i].val, val);
            }
            return &entry->data[i].val;
        }
    }
    struct hmap_entry mentry = {
        // same goes for keys
        .key = noalloc ? (char*)key : strdup(key)
    };
    if(noalloc) {
        mentry.val.type = val->type;
        mentry.val.as = val->as;
    } else {
        value_copy(&mentry.val, val);
        array_push(hmap->keys, mentry.key);
    }
    array_ptr_push(entry, mentry);
    if(entry->length == 1) hmap->occupied++;
    return &entry->data[entry->length-1].val;
}

void hmap_copy(struct hmap *dst, const struct hmap *src) {
    dst->occupied = src->occupied;
    dst->data = (a_hmap_buckets)array_init_n(a_hmap_entry, src->data.length);
    dst->keys = (a_hmap_keys)array_init_n(char*, src->keys.length);
    size_t key_idx = 0;
    for(size_t i = 0; i < src->data.length; i++) {
        dst->data.data[i] = (a_hmap_entry)array_init_n(struct hmap_entry, src->data.data[i].length);
        for(size_t j = 0; j < src->data.data[i].length; j++) {
            struct hmap_entry *entry = &dst->data.data[i].data[j];
            entry->key = strdup(src->data.data[i].data[j].key);
            value_copy(&entry->val, &src->data.data[i].data[j].val);
            dst->keys.data[key_idx++] = entry->key;
        }
    }
}

void hmap_del(struct hmap *hmap, const char *key) {
    a_hmap_entry *entry = &hmap->data.data[hmap_index(hmap, key)];
    if(entry->length == 1) {
        free(entry->data[0].key);
        //value_free(&entry->data[0].val);
        entry->length--;
        if(entry->length == 0) hmap->occupied--;
    } else {
        for(size_t i = 0; i < entry->length; i++)
            if(strcmp(entry->data[i].key, key) == 0) {
                // free entry
                free(entry->data[i].key);
                //value_free(&entry->data[i].val);
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
