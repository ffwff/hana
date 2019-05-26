extern crate libffi_sys;
use libffi_sys::*;
use std::ptr::{null, null_mut};
use libc::c_void;
use std::ffi::{CString, CStr};
use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::Value;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::record::Record;

struct FFI_Function {
    sym: Option<unsafe extern fn()>,
    cif: ffi_cif,
    argtypes: CArray<FFI_Type>,
    ffi_argtypes: CArray<*mut ffi_type>,
    rettype: FFI_Type,
    ffi_rettype: *mut ffi_type,
}

// #region ffi type
#[repr(i64)]
enum FFI_Type {
    UInt8,   Int8,
    UInt16,  Int16,
    UInt32,  Int32,
    UInt64,  Int64,
    Float32, Float64,
    Pointer,
    String,
    Void
}

impl FFI_Type {
    fn from_i64(value: i64) -> Self {
        match value {
            0  => FFI_Type::UInt8,
            1  => FFI_Type::Int8,
            2  => FFI_Type::UInt16,
            3  => FFI_Type::Int16,
            4  => FFI_Type::UInt32,
            5  => FFI_Type::Int32,
            6  => FFI_Type::UInt64,
            7  => FFI_Type::Int64,
            8  => FFI_Type::Float32,
            9  => FFI_Type::Float64,
            10 => FFI_Type::Pointer,
            11 => FFI_Type::String,
            12 => FFI_Type::Void,
            _ => panic!("wrong type!")
        }
    }

    unsafe fn to_libffi_type(&self) -> *mut ffi_type {
        match self {
            FFI_Type::UInt8   => &mut ffi_type_uint8,
            FFI_Type::Int8    => &mut ffi_type_sint8,
            FFI_Type::UInt16  => &mut ffi_type_uint16,
            FFI_Type::Int16   => &mut ffi_type_sint16,
            FFI_Type::UInt32  => &mut ffi_type_uint32,
            FFI_Type::Int32   => &mut ffi_type_sint32,
            FFI_Type::UInt64  => &mut ffi_type_uint64,
            FFI_Type::Int64   => &mut ffi_type_sint64,
            FFI_Type::Float32 => &mut ffi_type_float,
            FFI_Type::Float64 => &mut ffi_type_double,
            FFI_Type::Pointer => &mut ffi_type_pointer,
            FFI_Type::String  => &mut ffi_type_pointer,
            FFI_Type::Void    => &mut ffi_type_void
        }
    }
}
// #endregion

#[hana_function()]
fn constructor(name: Value::Str, argtypes: Value::Array, rettype: Value::Int) -> Value {
    // argtypes
    let argtypes = argtypes.as_ref();
    let mut inst_argtypes = CArray::reserve(argtypes.len());
    let mut ffi_argtypes = CArray::reserve(argtypes.len());
    for arg in argtypes.iter() {
        let ffi_type = FFI_Type::from_i64(arg.unwrap().int());
        ffi_argtypes.push(unsafe{ ffi_type.to_libffi_type() });
        inst_argtypes.push(ffi_type);
    }
    ffi_argtypes.push(null_mut());

    // rettype
    let rettype = FFI_Type::from_i64(rettype);
    let ffi_rettype = unsafe{ rettype.to_libffi_type() };

    // create
    let ffi_fn = unsafe {
        // cif
        let mut cif = std::intrinsics::init::<ffi_cif>();
        ffi_prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI,
            argtypes.len() as u32,
            ffi_rettype,
            ffi_argtypes.as_ptr() as *mut *mut ffi_type);

        // ffi fn
        let dl = libc::dlopen(null(), libc::RTLD_LAZY);
        FFI_Function {
            sym: {
                let sym = libc::dlsym(dl, name.as_ref().as_ptr() as *const i8);
                if sym.is_null() { None }
                else { Some(std::mem::transmute::<*mut c_void, unsafe extern fn()>(sym)) }
            },
            cif,
            argtypes: inst_argtypes,
            ffi_argtypes,
            rettype,
            ffi_rettype
        }
    };

    let rec = vm.malloc(Record::new());
    rec.as_mut().native_field = Some(Box::new(ffi_fn));
    Value::Record(rec)
}

#[hana_function()]
fn call(ffi_fn_rec: Value::Record, args: Value::Array) {
    let field = ffi_fn_rec.as_ref().native_field.as_ref().unwrap();
    let mut ffi_fn = field.downcast_ref::<FFI_Function>().clone().unwrap();

    use libc::c_void;
    use std::any::Any;
    use std::convert::TryInto;
    use std::mem::transmute;

    let mut managed_strs : Vec<Box<CStr>> = Vec::new();
    let mut aref : CArray<*mut c_void> =
        CArray::reserve(args.as_ref().len());
    let mut argtypes_iter = ffi_fn.argtypes.iter();
    let slice = args.as_mut().as_mut_slice();
    for arg in slice {
        let argtype = &argtypes_iter.next().unwrap();
        unsafe {
            let ptr = transmute::<&u64, *mut c_void>(&arg.data);
            #[allow(safe_packed_borrows)]
            match argtype {
                // TODO make this clearer
                FFI_Type::String => {
                    let cstr : Box<CStr> = CString::new(arg.unwrap().string().clone()).unwrap().into_boxed_c_str();
                    aref.push(transmute::<*const libc::c_char, *mut c_void>(cstr.as_ptr()));
                    managed_strs.push(cstr);
                },
                // primitive types
                _ => { aref.push(transmute::<&mut u64, *mut c_void>(&mut arg.data)); },
            }
        }
    }

    match &ffi_fn.rettype {
        FFI_Type::UInt8 | FFI_Type::Int8 | FFI_Type::UInt16 | FFI_Type::Int16  | FFI_Type::UInt32 | FFI_Type::Int32 | FFI_Type::UInt64 | FFI_Type::Int64
            => unsafe {
                let mut rvalue = std::intrinsics::uninit::<i64>();
                ffi_call(transmute::<&ffi_cif, *mut ffi_cif>(&ffi_fn.cif), ffi_fn.sym, transmute::<&i64, *mut c_void>(&rvalue), &mut aref.as_mut_ptr());
                Value::Int(rvalue)
            },
        _ => unimplemented!()
        /*
        FFI_Type::Float32 => &mut ffi_type_float,
        FFI_Type::Float64 => &mut ffi_type_double,
        FFI_Type::Pointer => &mut ffi_type_pointer,
        FFI_Type::String  => &mut ffi_type_pointer,
        FFI_Type::Void    => &mut ffi_type_void*/
    }
}