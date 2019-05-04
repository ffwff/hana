use std::ptr::null_mut;

#[repr(C)]
struct CArray<T> {
    data: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> CArray<T> {
    fn new_nil() -> CArray<T> {
        CArray::<T> {
            data: null_mut(),
            len: 0,
            capacity: 0
        }
    }
}

//
#[repr(C)]
pub struct Value {
    reserved : u64,
    r#type : u8
}

//
#[repr(C)]
pub struct Vm {
    ip : u32,
    localenv : *mut i32,
    globalenv : *mut i32,
    eframe : *mut i32,
    code : CArray<u8>,
    stack : CArray<Value>,
    dstr: *mut i32,
    dint: *mut i32,
    dfloat: *mut i32,
    darray: *mut i32,
    error: bool
}

#[link(name="hana", kind="static")]
extern "C" {
    fn vm_init(vm: *mut Vm);
    fn vm_free(vm: *mut Vm);
}

impl Vm {
    pub fn new() -> Vm {
        let mut vm = Vm{
            ip: 0,
            localenv: null_mut(),
            globalenv: null_mut(),
            eframe: null_mut(),
            code: CArray::new_nil(),
            stack: CArray::new_nil(),
            dstr: null_mut(),
            dint: null_mut(),
            dfloat: null_mut(),
            darray: null_mut(),
            error: false,
        };
        unsafe { vm_init(&mut vm); }
        vm
    }
}

impl std::ops::Drop for Vm {
    fn drop(&mut self) {
        unsafe { vm_free(self); }
    }
}