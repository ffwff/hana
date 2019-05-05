use std::ptr::null_mut;
use std::ffi::CString;
use std::ffi::CStr;
extern crate libc;
use super::carray::CArray;
use super::stringheader::StringHeader;

//
#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
#[derive(PartialEq)]
enum _valueType {
    TYPE_NIL        = 0,
    TYPE_INT        = 1,
    TYPE_FLOAT      = 2,
    TYPE_NATIVE_FN  = 3,
    TYPE_FN         = 4,
    TYPE_STR        = 5,
    TYPE_DICT       = 6,
    TYPE_ARRAY      = 7,
    TYPE_NATIVE_OBJ = 8,
}

#[derive(PartialEq, Debug)]
pub enum Value {
    Nil,
    Int(i64),
    Float(f64),
    NativeFn,
    Fn,
    Str(&'static str),
    Dict,
    Array,
    NativeObj
}

#[repr(C)]
pub struct NativeValue {
    data : u64,
    r#type : _valueType
}

impl NativeValue {

    pub fn unwrap(&self) -> Value {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        match &self.r#type {
_valueType::TYPE_NIL        => Value::Nil,
_valueType::TYPE_INT        => unsafe { Value::Int(transmute::<u64, i64>(self.data)) },
_valueType::TYPE_FLOAT      => unsafe { Value::Float(transmute::<u64, f64>(self.data)) },
_valueType::TYPE_NATIVE_FN  => Value::NativeFn,
_valueType::TYPE_FN         => Value::Fn,
_valueType::TYPE_STR        => unsafe {
        use std::mem::size_of;
        let cstr = CStr::from_ptr(transmute::<u64,*const libc::c_char>(self.data).add(size_of::<StringHeader>()));
        Value::Str(cstr.to_str().unwrap())
    },
_valueType::TYPE_DICT       => Value::Dict,
_valueType::TYPE_ARRAY      => Value::Array,
_valueType::TYPE_NATIVE_OBJ => Value::NativeObj,
        }
    }

}

//
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum VmOpcode {
    OP_HALT,
    // stack manip
    OP_PUSH8, OP_PUSH16, OP_PUSH32, OP_PUSH64,
    OP_PUSH_NIL, OP_PUSHSTR, OP_PUSHF32, OP_PUSHF64,
    OP_POP,
    // arith
    OP_ADD, OP_SUB, OP_MUL, OP_DIV, OP_MOD,
    // logic
    OP_AND, OP_OR,
    // unary
    OP_NEGATE, OP_NOT,
    // comparison
    OP_LT, OP_LEQ, OP_GT, OP_GEQ,
    OP_EQ, OP_NEQ,
    // variables
    OP_ENV_NEW,
    OP_SET_LOCAL, OP_SET_LOCAL_FUNCTION_DEF, OP_GET_LOCAL,
    OP_SET_LOCAL_UP, OP_GET_LOCAL_UP,
    OP_SET_GLOBAL, OP_GET_GLOBAL,
    OP_DEF_FUNCTION_PUSH,
    // flow control
    OP_JMP, OP_JCOND, OP_JNCOND, OP_CALL, OP_RET,
    // dictionary
    OP_DICT_NEW, OP_MEMBER_GET, OP_MEMBER_GET_NO_POP,
    OP_MEMBER_SET, OP_DICT_LOAD, OP_ARRAY_LOAD,
    OP_INDEX_GET, OP_INDEX_SET,
    // exceptions
    OP_TRY, OP_RAISE, OP_EXFRAME_RET,
    // tail calls
    OP_RETCALL
}

#[repr(C)]
pub struct Vm {
    // TODO: fill in all these *mut i32
    pub ip : u32,
    pub localenv : *mut i32,
    pub eframe : *mut i32,
    pub code : CArray<VmOpcode>,
    pub stack : CArray<NativeValue>,
    pub dstr: *mut i32,
    pub dint: *mut i32,
    pub dfloat: *mut i32,
    pub darray: *mut i32,
    pub error: bool
}

#[link(name="hana", kind="static")]
extern "C" {
    fn vm_init(vm: *mut Vm);
    fn vm_free(vm: *mut Vm);
    fn vm_execute(vm: *mut Vm);

    fn vm_code_reserve(vm: *mut Vm, sz: usize);
    fn vm_code_push8  (vm: *mut Vm, n : u8);
    fn vm_code_push16 (vm: *mut Vm, n : u16);
    fn vm_code_push32 (vm: *mut Vm, n : u32);
    fn vm_code_push64 (vm: *mut Vm, n : u64);
    fn vm_code_pushstr(vm: *mut Vm, s : *const libc::c_char);
    fn vm_code_pushf32(vm: *mut Vm, n : f32);
    fn vm_code_pushf64(vm: *mut Vm, n : f64);
}

impl Vm {
    pub fn new() -> Vm {
        let mut vm = Vm{
            ip: 0,
            localenv: null_mut(),
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

    pub fn execute(&mut self) {
        unsafe { vm_execute(self); }
    }

    // pushes
    pub fn cpush8(&mut self, n : u8) { unsafe { vm_code_push8(self, n); } }
    pub fn cpush16(&mut self, n : u16) { unsafe { vm_code_push16(self, n); } }
    pub fn cpush32(&mut self, n : u32) { unsafe { vm_code_push32(self, n); } }
    pub fn cpush64(&mut self, n : u64) { unsafe { vm_code_push64(self, n); } }
    pub fn cpushf32(&mut self, n : f32) { unsafe { vm_code_pushf32(self, n); } }
    pub fn cpushf64(&mut self, n : f64) { unsafe { vm_code_pushf64(self, n); } }
    pub fn cpushs<T : Into<Vec<u8>>>(&mut self, s : T) {
        let cstr = CString::new(s).expect("can't turn to cstring");
        unsafe { vm_code_pushstr(self, cstr.as_ptr()); }
    }
}

impl std::ops::Drop for Vm {
    fn drop(&mut self) {
        unsafe { vm_free(self); }
    }
}
