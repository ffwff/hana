#include <stdio.h>
#include <dlfcn.h>
#include <ffi.h>
#include "hanayo.h"
#include "vm/src/string_.h"
#include "vm/src/array_obj.h"

#define fn(name) void hanayo::ffi::name(struct vm *vm, int nargs)

int debug(const char *ptr) { return 1; }
typedef array(value::value_type) a_value_type;
typedef void (*ffi_fnptr)();

struct ffi_function {
    ffi_fnptr sym;
    ffi_cif cif;
    a_value_type argtypes;
    ffi_type **ffi_argtypes;
    value::value_type rettype;
};

static void ffi_function_free(void *ffn_) {
    auto ffn = (struct ffi_function *)ffn_;
    array_free(ffn->argtypes);
    free(ffn_);
}

fn(function) { // cffi_function("name", [argtypes,...], rettype)
    struct value val;
    void *dl = nullptr;
    struct ffi_function *ffn = (struct ffi_function *)malloc(sizeof(struct ffi_function));
    ffn->argtypes = {
        .data = (value::value_type*)calloc(1, sizeof(value::value_type)),
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
        assert(v.type == value::TYPE_DICT);
        auto dict = v.as.dict;
        if(dict == vm->dstr) {
            ffn->ffi_argtypes[i] = &ffi_type_pointer;
            array_push(ffn->argtypes, value::TYPE_STR);
        } else if(dict == vm->dint) {
            ffn->ffi_argtypes[i] = &ffi_type_sint64;
            array_push(ffn->argtypes, value::TYPE_INT);
        } else if(dict == vm->dfloat) {
            ffn->ffi_argtypes[i] = &ffi_type_double;
            array_push(ffn->argtypes, value::TYPE_FLOAT);
        } else {
            assert(0);
        }
        value_free(&v);
    }
    ffn->ffi_argtypes[ffi_nargs] = nullptr;

    // ret type
    val = _arg(vm, value::TYPE_DICT);
    ffi_type *rettype;
    const auto dict = val.as.dict;
    if(dict == vm->dstr) {
        rettype = &ffi_type_pointer;
        ffn->rettype = value::TYPE_STR;
    } else if(dict == vm->dint) {
        rettype = &ffi_type_sint64;
        ffn->rettype = value::TYPE_INT;
    } else if(dict == vm->dfloat) {
        rettype = &ffi_type_double;
        ffn->rettype = value::TYPE_FLOAT;
    } else {
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
    val = _arg(vm, value::TYPE_NATIVE_OBJ);
    struct ffi_function *ffn = (struct ffi_function *)(val.as.native->data);
    nargs--;

    // arguments
    assert((size_t)nargs == ffn->argtypes.length);
    void *aptr[nargs]; // maybe a HACK to store pointers to strings
    // libffi apparently gets the pointer in the dereferenced key of avalues
    // essentially a double pointer
    void *avalues[nargs];
    struct value avalues_v[nargs];
    for(int64_t i = 0; i < nargs; i++) {
        struct value val;
        if(ffn->argtypes.data[i] == value::TYPE_STR) {
            val = _arg(vm, value::TYPE_STR);
            aptr[i] = string_data(val.as.str);
            avalues[i] = &aptr[i];
        } else if(ffn->argtypes.data[i] == value::TYPE_INT) {
            // TODO
        } else if(ffn->argtypes.data[i] == value::TYPE_FLOAT) {
            // TODO
        } else {
            assert(0);
        }
        avalues_v[i] = val;
    }

    // call and ret
    struct value retval;
    if(ffn->rettype == value::TYPE_INT) {
        int64_t n;
        ffi_call(&ffn->cif, ffn->sym, &n, avalues);
        value_int(&retval, n);
    } else {
        assert(0);
    }

    array_push(vm->stack, retval);

    // cleanup args
    for(int64_t i = 0; i < nargs; i++)
        value_free(avalues_v);

}
