#include <stdio.h>
#include <limits.h>
#include "hanayo.h"
#include "vm/src/string_.h"

#define fn(name) void hanayo::name(struct vm *vm, int nargs)

fn(print) {
    int written = 0;
    while(nargs--) {
        struct value val = array_top(vm->stack);
        const auto s = hanayo::_to_string(val);
        written += fputs(s, stdout);
        free(s);
        value_free(&val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
    array_push(vm->stack, val);
}

fn(input) {
    char *line = nullptr;
    size_t n = 0;
    getline(&line, &n, stdin);
    struct value val;
    value_str(&val, line); free(line);
    array_push(vm->stack, val);
}

// files
fn(fopen) {
    // path : str, mode : str
    assert(nargs == 2);
    auto path = _arg(vm, value::TYPE_STR);
    auto mode = _arg(vm, value::TYPE_STR);

    struct value val;
    value_native_obj(&val,
                     ::fopen(string_data(path.as.str), string_data(mode.as.str)),
                     [](void *data) {
                        fclose((FILE*)data);
                     });
    value_free(&path);
    value_free(&mode);
    array_push(vm->stack, val);
}

fn(fread) {
    // file : FILE*, chars: int -> str
    //assert(nargs == 2);
    if(nargs == 1) {
        auto val = _arg(vm, value::TYPE_NATIVE_OBJ);

        auto f = (FILE*)val.as.native->data;
        fseek(f, 0, SEEK_END);
        auto size = ftell(f);
        fseek(f, 0, SEEK_SET);

        char *buf = (char*)malloc(size+1);
        size_t n = fread(buf, 1, size, f);
        buf[n] = 0;
        struct value s;
        value_str(&s, buf);
        free(buf);
        value_free(&val);
        array_push(vm->stack, s);
    } else if(nargs == 2) {
        auto val = _arg(vm, value::TYPE_NATIVE_OBJ);
        auto chars = _arg(vm, value::TYPE_INT);

        char *buf = (char*)malloc(chars.as.integer+1);
        size_t n = fread(buf, 1, chars.as.integer, (FILE*)val.as.native->data);
        buf[n] = 0;
        struct value s;
        value_str(&s, buf);
        free(buf);
        value_free(&val);
        array_push(vm->stack, s);
    } else {
        assert(0);
    }
}

fn(fwrite) {
    // file : FILE*, buf: str -> size_t
    assert(nargs == 2);
    auto val = _arg(vm, value::TYPE_NATIVE_OBJ);
    auto buf = _arg(vm, value::TYPE_STR);

    struct value s;
    value_int(&s, fwrite(string_data(buf.as.str), 1,
              buf.as.str->length, (FILE*)val.as.native->data));
    value_free(&val);
    value_free(&buf);
    array_push(vm->stack, s);

}

fn(realpath) {
    assert(nargs == 1);
    auto path = _arg(vm, value::TYPE_STR);
    char actualpath [PATH_MAX+1];
    char *ptr = ::realpath(string_data(path.as.str), actualpath);
    value_free(&path);
    if(ptr == nullptr) {
        array_push(vm->stack, {0});
        return;
    }
    value_str(&path, ptr);
    array_push(vm->stack, path);

}
