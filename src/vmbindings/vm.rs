use std::ptr::null_mut;
use std::ffi::CString;
extern crate libc;

use super::carray::CArray;
use super::chmap::CHashMap;
use super::record::Record;
use super::function::Function;
use super::cnativeval::NativeValue;
use super::exframe::ExFrame;
use super::env::Env;
pub use super::value::Value;

const CALL_STACK_SIZE : usize = 512;

//
#[repr(u8)]
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum VmError {
    ERROR_NO_ERROR = 0,
    ERROR_OP_ADD,
    ERROR_OP_SUB,
    ERROR_OP_MUL,
    ERROR_OP_DIV,
    ERROR_OP_MOD,
    ERROR_OP_AND,
    ERROR_OP_OR,
    ERROR_OP_LT,
    ERROR_OP_LEQ,
    ERROR_OP_GT,
    ERROR_OP_GEQ,
    ERROR_OP_EQ,
    ERROR_OP_NEQ,
    ERROR_UNDEFINED_GLOBAL_VAR,
    ERROR_RECORD_NO_CONSTRUCTOR,
    ERROR_CONSTRUCTOR_NOT_FUNCTION,
    ERROR_MISMATCH_ARGUMENTS,
    ERROR_EXPECTED_CALLABLE,
    ERROR_CANNOT_ACCESS_NIL,
    ERROR_CANNOT_ACCESS_NON_RECORD,
    ERROR_CANNOT_ACCESS_NON_STRING,
    ERROR_KEY_NON_INT,
    ERROR_RECORD_KEY_NON_STRING,
    ERROR_UNBOUNDED_ACCESS,
    ERROR_EXPECTED_RECORD_ARRAY,
    ERROR_CASE_EXPECTS_DICT,
    ERROR_UNHANDLED_EXCEPTION,
}

impl std::fmt::Display for VmError {

    #[allow(non_snake_case)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        VmError::ERROR_OP_ADD                   => write!(f, "Invalid arguments for addition"),
        VmError::ERROR_OP_SUB                   => write!(f, "Invalid arguments for subtraction"),
        VmError::ERROR_OP_MUL                   => write!(f, "Invalid arguments for multiplication"),
        VmError::ERROR_OP_DIV                   => write!(f, "Invalid arguments for division"),
        VmError::ERROR_OP_MOD                   => write!(f, "Invalid arguments for modulo"),
        VmError::ERROR_OP_AND                   => write!(f, "Invalid arguments for logical and"),
        VmError::ERROR_OP_OR                    => write!(f, "Invalid arguments for logical or"),
        VmError::ERROR_OP_LT                    => write!(f, "Invalid arguments for less than"),
        VmError::ERROR_OP_LEQ                   => write!(f, "Invalid arguments for less than or equal to"),
        VmError::ERROR_OP_GT                    => write!(f, "Invalid arguments for greater than"),
        VmError::ERROR_OP_GEQ                   => write!(f, "Invalid arguments for greater than or equal to"),
        VmError::ERROR_OP_EQ                    => write!(f, "Invalid arguments for equality"),
        VmError::ERROR_OP_NEQ                   => write!(f, "Invalid arguments for inequality"),
        VmError::ERROR_UNDEFINED_GLOBAL_VAR     => write!(f, "Global variable is not defined"),
        VmError::ERROR_RECORD_NO_CONSTRUCTOR    => write!(f, "Cannot call record that has no constructor"),
        VmError::ERROR_CONSTRUCTOR_NOT_FUNCTION => write!(f, "Cannot call record with non-function constructor"),
        VmError::ERROR_MISMATCH_ARGUMENTS       => write!(f, "Argument mismatch"),
        VmError::ERROR_EXPECTED_CALLABLE        => write!(f, "Expected value to be callable (record or function)"),
        VmError::ERROR_CANNOT_ACCESS_NIL        => write!(f, "Cannot access property of nil that is not \"prototype\""),
        VmError::ERROR_CANNOT_ACCESS_NON_RECORD => write!(f, "Cannot access non record"),
        VmError::ERROR_CANNOT_ACCESS_NON_STRING => write!(f, "Cannot access non string"),
        VmError::ERROR_KEY_NON_INT              => write!(f, "Index must be an integer value"),
        VmError::ERROR_RECORD_KEY_NON_STRING    => write!(f, "Record key must be an string value"),
        VmError::ERROR_UNBOUNDED_ACCESS         => write!(f, "Unbounded access"),
        VmError::ERROR_EXPECTED_RECORD_ARRAY    => write!(f, "Expected record or array"),
        VmError::ERROR_CASE_EXPECTS_DICT        => write!(f, "case statement expects a dictionary as handler type"),
        VmError::ERROR_UNHANDLED_EXCEPTION      => write!(f, "Unhandled exception"),
        _ => unreachable!()
        }
    }

}

#[repr(C)]
pub struct Vm {
    pub ip        : u32, // current instruction pointer
    pub localenv  : *mut Env,
    // pointer to current stack frame
    localenv_bp   : *mut Env, // rust owns this, so drop it pls
    // pointer to start of pool of stack frames
    pub globalenv : *mut CHashMap,
    // global environment, all unscoped variables/variables
    // starting with '$' should be stored here
    pub exframes  : CArray<ExFrame>, // exception frame
    pub code      : CArray<VmOpcode>, // where all the code is
    pub stack     : CArray<NativeValue>, // stack

    // prototype types for primitive values
    pub dstr      : *mut Record,
    pub dint      : *mut Record,
    pub dfloat    : *mut Record,
    pub darray    : *mut Record,

    pub error     : VmError
    // whether the interpreter raised an unhandled error
}

#[link(name="hana", kind="static")]
#[allow(improper_ctypes)]
extern "C" {
    fn vm_init(vm: *mut Vm);
    fn vm_free(vm: *mut Vm);
    fn vm_execute(vm: *mut Vm);
    fn vm_print_stack(vm: *const Vm);
    fn vm_call(vm: *mut Vm, fun: NativeValue, args: CArray<NativeValue>)
        -> NativeValue;

    fn vm_code_push8  (vm: *mut Vm, n : u8);
    fn vm_code_push16 (vm: *mut Vm, n : u16);
    fn vm_code_push32 (vm: *mut Vm, n : u32);
    fn vm_code_push64 (vm: *mut Vm, n : u64);
    fn vm_code_pushstr(vm: *mut Vm, s : *const libc::c_char);
    fn vm_code_pushf32(vm: *mut Vm, n : f32);
    fn vm_code_pushf64(vm: *mut Vm, n : f64);
    fn vm_code_fill(vm: *mut Vm, pos : u32, len : u32);
    fn vm_code_fill16(vm: *mut Vm, pos : u32, len : u16);
}

impl Vm {
    pub fn new() -> Vm {
        let mut vm = Vm{
            ip: 0,
            localenv: null_mut(),
            localenv_bp: {
                use std::alloc::{alloc, Layout};
                use std::mem::size_of;
                let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
                unsafe { alloc(layout.unwrap()) as *mut Env }
            },
            globalenv: null_mut(),
            exframes: CArray::new(),
            code: CArray::new_nil(),
            stack: CArray::new_nil(),
            dstr: null_mut(),
            dint: null_mut(),
            dfloat: null_mut(),
            darray: null_mut(),
            error: VmError::ERROR_NO_ERROR,
        };
        unsafe { vm_init(&mut vm); }
        vm
    }

    pub fn print_stack(&self) {
        unsafe { vm_print_stack(self); }
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

    pub fn cfill_label(&mut self, pos: usize, label: usize) {
        unsafe{ vm_code_fill(self, pos as u32, label as u32); }
    }
    pub fn cfill_label16(&mut self, pos: usize, label: u16) {
        unsafe{ vm_code_fill16(self, pos as u32, label); }
    }

    // globals
    pub fn global(&mut self) -> &mut CHashMap {
        if self.globalenv.is_null() { panic!("accessing nil ptr"); }
        unsafe{ &mut *self.globalenv }
    }

    // gc
    pub fn mark(&mut self) {
        // globalenv
        let globalenv = self.global();
        for (_, val) in globalenv.iter() {
            val.mark();
        }
        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            val.mark();
        }
        // call stack
        if !self.localenv.is_null() { unsafe {
            let mut env = self.localenv_bp;
            while env != self.localenv {
                for i in 0..(*env).nslots {
                    let val = (*env).get(i);
                    val.mark();
                }
                env = env.add(1);
            }
            env = self.localenv;
            for i in 0..(*env).nslots {
                let val = (*env).get(i);
                val.mark();
            }
        } }
    }

    // call stack
    pub fn enter_env(&mut self, fun: &'static Function) {
        if self.localenv.is_null() {
            self.localenv = self.localenv_bp;
        } else {
            if unsafe {
                self.localenv.offset_from(self.localenv_bp) > (CALL_STACK_SIZE as isize)
            } { panic!("maximum stack depth exceeded"); }
            self.localenv = unsafe{ self.localenv.add(1) };
        }
        {
            // NOTE: std::mem::replace causes memory corruption
            // when replacing unallocated stack env with current env
            use std::ptr::copy_nonoverlapping;
            let env = Env::new(self.ip, fun.bound, fun.nargs);
            unsafe { copy_nonoverlapping(&env, self.localenv, 1); }
        }
        self.ip = fun.ip;
    }

    pub fn enter_env_tail(&mut self, fun: &'static Function) {
        let env = unsafe{ &mut *self.localenv };
        env.nargs = fun.nargs;
        env.lexical_parent = fun.bound;
        self.ip = fun.ip;
    }

    pub fn leave_env(&mut self) {
        // we don't check for env leaving
        // this must be non-null
        unsafe {
            self.ip = (*self.localenv).retip;
            if self.localenv == self.localenv_bp {
                self.localenv = null_mut();
            } else {
                self.localenv = self.localenv.sub(1);
            }
        }
    }

    // exceptions
    pub fn enter_exframe(&mut self) -> &mut ExFrame {
        self.exframes.push(ExFrame::new(self.localenv, self.stack.len()-1));
        self.exframes.top_mut()
    }
    pub fn leave_exframe(&mut self) {
        self.exframes.pop();
    }
    pub fn raise(&mut self) -> bool {
        if self.exframes.len() == 0 { return false; }
        let val = self.stack.top().unwrap();
        let mut i = self.exframes.len();
        while i != 0 {
            let exframe = &self.exframes[i-1];
            if let Some(handler) = exframe.get_handler(self, &val) {
                self.ip = handler.ip;
                if handler.nargs == 0 {
                    self.stack.pop();
                }
                return true;
            }
            i -= 1;
        }
        false
    }

    // functions
    pub fn call(&mut self, fun: NativeValue, args: CArray<NativeValue>) -> NativeValue {
        unsafe{ vm_call(self, fun, args) }
    }
}

impl std::ops::Drop for Vm {
    fn drop(&mut self) {
        unsafe {
            use std::alloc::{dealloc, Layout};
            use std::mem;

            if !self.localenv.is_null() {
                let mut env = self.localenv_bp;
                while env != self.localenv {
                    mem::drop(env);
                    env = env.add(1);
                }
                mem::drop(self.localenv);
            }
            let layout = Layout::from_size_align(mem::size_of::<Env>() * CALL_STACK_SIZE, 4);
            dealloc(self.localenv_bp as *mut u8, layout.unwrap());

            self.exframes.drop();

            vm_free(self);

        }
    }
}
