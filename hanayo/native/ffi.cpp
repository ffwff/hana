#include <stdio.h>
#include <dlfcn.h>
#include <ffi.h>
#include "hanayo.h"
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"
#include "vm/src/dict.h"

#define fn(name) void hanayo::ffi::name(struct vm *vm, int nargs)

//int debug(float ptr) { return 1; }
int debug(double ptr) { return 1; }
typedef array(hanayo::ffi::type) a_ffi_type;
typedef void (*ffi_fnptr)();

struct ffi_function {
    ffi_fnptr sym;
    ffi_cif cif;
    a_ffi_type argtypes;
    ffi_type **ffi_argtypes;
    hanayo::ffi::type rettype;
};

static void ffi_function_free(void *ffn_) {
    auto ffn = (struct ffi_function *)ffn_;
    array_free(ffn->argtypes);
    free(ffn->ffi_argtypes);
    free(ffn_);
}

fn(function) { // cffi_function("name", [argtypes,...], rettype)
    struct value val;
    void *dl = nullptr;
    struct ffi_function *ffn = (struct ffi_function *)malloc(sizeof(struct ffi_function));
    ffn->argtypes = {
        .data = (hanayo::ffi::type*)calloc(1, sizeof(hanayo::ffi::type)),
        .length = 0,
        .capacity = 1
    };

    if(nargs == 4) { // TODO
        val = _arg(vm, value::TYPE_STR);
        dl = dlopen(string_data(val.as.str), RTLD_LAZY);
        value_free(&val);
    } else if(nargs == 3) {
        dl = dlopen(nullptr, RTLD_LAZY);
    } else {
        assert(0);
    }

    // function name
    val = _arg(vm, value::TYPE_STR);
    //ffn->sym = (ffi_fnptr)debug;
    ffn->sym = (ffi_fnptr)dlsym(dl, string_data(val.as.str));
    assert(ffn->sym != nullptr);
    value_free(&val);

    // argtypes
    val = _arg(vm, value::TYPE_ARRAY);
    size_t ffi_nargs = val.as.array->data.length;
    ffn->ffi_argtypes = (ffi_type**)malloc(sizeof(ffi_type*)*(ffi_nargs+1));
    for(size_t i = 0; i < ffi_nargs; i++) {
        auto v = val.as.array->data.data[i];
        assert(v.type == value::TYPE_INT);
        hanayo::ffi::type t = (hanayo::ffi::type)v.as.integer;
        switch(t) {
        case String:
            ffn->ffi_argtypes[i] = &ffi_type_pointer;
            array_push(ffn->argtypes, String);
            break;
        // primitives (numbers/floats)
#define X(x,y) \
        case x: \
            ffn->ffi_argtypes[i] = &ffi_type_ ## y; \
            array_push(ffn->argtypes, x); \
            break;
        X(UInt8,  uint8)  X(Int8 , sint8)
        X(UInt16, uint16) X(Int16, sint16)
        X(UInt32, uint32) X(Int32, sint32)
        X(UInt64, uint64) X(Int64, sint64)
        X(Float32, float)
        X(Float64, double)
        X(UChar,  uchar)  X(Char,  schar)
        X(UShort, ushort) X(Short, sshort)
        X(ULong,  ulong)  X(Long,  slong)
        X(Pointer, pointer)
        X(Void, void)
#undef X
        default:
            assert(0);
        }
        value_free(&v);
    }
    value_free(&val);
    ffn->ffi_argtypes[ffi_nargs] = nullptr;

    // ret type
    val = _arg(vm, value::TYPE_INT);
    const hanayo::ffi::type t = (hanayo::ffi::type)val.as.integer;
    ffi_type *rettype;
    switch(t) {
    case String:
        rettype = &ffi_type_pointer;
        ffn->rettype = String;
        break;
#define X(x,y) \
    case x: \
        rettype = &ffi_type_ ## y; \
        ffn->rettype = x; \
        break;
    X(UInt8,  uint8)  X(Int8 , sint8)
    X(UInt16, uint16) X(Int16, sint16)
    X(UInt32, uint32) X(Int32, sint32)
    X(UInt64, uint64) X(Int64, sint64)
    X(Float32, float)
    X(Float64, double)
    X(UChar,  uchar)  X(Char,  schar)
    X(UShort, ushort) X(Short, sshort)
    X(ULong,  ulong)  X(Long,  slong)
    X(Pointer, pointer)
    X(Void, void)
#undef X
    default:
        assert(0);
    }
    value_free(&val);

    // setup
    ffi_prep_cif(&ffn->cif, FFI_DEFAULT_ABI, ffi_nargs, rettype, ffn->ffi_argtypes);
    value_native_obj(&val, ffn, ffi_function_free);
    array_push(vm->stack, val);

    // TODO cleanup dlsym
}

fn(call) {
    struct value val;

    // ffn
    auto ffnv = _arg(vm, value::TYPE_NATIVE_OBJ);
    struct ffi_function *ffn = (struct ffi_function *)(ffnv.as.native->data);
    nargs--;

    // arguments
    assert((size_t)nargs == ffn->argtypes.length);
    void *aptr[nargs]; // HACK: arguments store for strings
    // libffi apparently gets the pointer in the dereferenced key of avalues
    // essentially a double pointer
    void *avalues[nargs];
    struct value avalues_v[nargs];
    for(int64_t i = 0; i < nargs; i++) {
        struct value val;
        switch (ffn->argtypes.data[i]) {
        case String:
            val = _arg(vm, value::TYPE_STR);
            avalues_v[i] = val;
            aptr[i] = string_data(val.as.str);
            avalues[i] = &aptr[i];
            break;
#define X(x,z) \
        case x: \
            val = _arg(vm, value::TYPE_INT); \
            avalues_v[i].type = value::TYPE_INT; \
            avalues_v[i].as.integer = (z)val.as.integer; \
            avalues[i] = &avalues_v[i].as.integer; \
            break;
        X(UInt8,  uint8_t)  X(Int8 , int8_t)
        X(UInt16, uint16_t) X(Int16, int16_t)
        X(UInt32, uint32_t) X(Int32, int32_t)
        X(UInt64, uint64_t) X(Int64, int64_t)
        X(UChar,  uint8_t)  X(Char,  int8_t)
        X(UShort, uint16_t) X(Short, int16_t)
        X(ULong,  uint64_t) X(Long,  int64_t)
        X(Pointer, intptr_t)
#undef X
#define X(x,z) \
        case x: \
            val = _arg(vm, value::TYPE_FLOAT); \
            avalues_v[i].type = value::TYPE_FLOAT; \
            avalues_v[i].as.floatp = (z)val.as.floatp; \
            avalues[i] = &avalues_v[i].as.floatp; \
            break;
        X(Float32, float)
        X(Float64, double)
#undef X
        //X(Void, void)
        default:
            assert(0);
            break;
        }
    }

    // call and ret
    struct value retval;
    switch(ffn->rettype) {
    case String: {
        const char *s;
        ffi_call(&ffn->cif, ffn->sym, &s, avalues);
        if(s == nullptr) value_str(&retval, "");
        else value_str(&retval, s);
        break;
    }
#define X(x,y) \
    case x: { \
        y n; \
        ffi_call(&ffn->cif, ffn->sym, &n, avalues); \
        value_int(&retval, n); \
        break; }
    X(UInt8,  uint8_t)  X(Int8 , int8_t)
    X(UInt16, uint16_t) X(Int16, int16_t)
    X(UInt32, uint32_t) X(Int32, int32_t)
    X(UInt64, uint64_t) X(Int64, int64_t)
    X(UChar,  uint8_t)  X(Char,  int8_t)
    X(UShort, uint16_t) X(Short, int16_t)
    X(ULong,  uint64_t) X(Long,  int64_t)
    X(Pointer, intptr_t)
#undef X
#define X(x,y) \
    case x: { \
        y n; \
        ffi_call(&ffn->cif, ffn->sym, &n, avalues); \
        value_float(&retval, n); \
        break; }
    X(Float32, float)
    X(Float64, double)
#undef X
    default:
        assert(0);
        break;
    }

    array_push(vm->stack, retval);

    // cleanup args
    for(int64_t i = 0; i < nargs; i++)
        value_free(&avalues_v[i]);
    value_free(&ffnv);

}

// RC Pointer
struct value hanayo::ffi::rcpointer::prototype;

#undef fn
#define fn(name) void hanayo::ffi::rcpointer::name(struct vm *vm, int nargs)
fn(constructor) {
    assert(nargs == 2);

    struct value ptrv = _arg(vm, value::TYPE_INT);
    intptr_t ptr = ptrv.as.integer;

    struct value ffnv = _arg(vm, value::TYPE_NATIVE_OBJ);
    auto ffn = (struct ffi_function *)ffnv.as.native->data;
    // free function must be of type void (*fn)(void *data);
    assert(ffn->argtypes.length == 1 && ffn->argtypes.data[0] == hanayo::ffi::type::Pointer);
    assert(ffn->rettype == hanayo::ffi::type::Void);
    value_free(&ffnv);

    struct value val; value_dict(&val);
    {
        struct value nval; value_native_obj(&nval, (void*)ptr, (native_obj_free_fn)ffn->sym);
        dict_set(val.as.dict, "free_fn", &nval);
        value_free(&nval);
    }
    {
        struct value nval; value_int(&nval, ptr);
        dict_set(val.as.dict, "pointer!", &nval);
    }
    dict_set(val.as.dict, "prototype", &hanayo::ffi::rcpointer::prototype);
    //;
    array_push(vm->stack, val);
}