use std::ptr::null_mut;
use std::mem::ManuallyDrop;
use std::ffi::CString;
use std::path::Path;
extern crate libc;

use super::carray::CArray;
use super::chmap::CHashMap;
use super::record::Record;
use super::function::Function;
use super::cnativeval::NativeValue;
use super::exframe::ExFrame;
use super::env::Env;
pub use super::value::Value;
use super::vmerror::VmError;
use super::gc::unpin;
use crate::compiler::Compiler;

const CALL_STACK_SIZE : usize = 512;

//
#[repr(u8)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum VmOpcode {
    OP_HALT,
    // stack manip
    OP_PUSH8, OP_PUSH16, OP_PUSH32, OP_PUSH64,
    OP_PUSH_NIL, OP_PUSHSTR, OP_PUSHF64,
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
    // matching
    OP_OF /* type matching */,
    // variables
    OP_ENV_NEW,
    OP_SET_LOCAL, OP_SET_LOCAL_FUNCTION_DEF, OP_GET_LOCAL,
    OP_GET_LOCAL_UP,
    OP_SET_GLOBAL, OP_GET_GLOBAL,
    OP_DEF_FUNCTION_PUSH,
    // flow control
    OP_JMP, OP_JMP_LONG, OP_JCOND, OP_JNCOND, OP_CALL, OP_RET,
    // dictionary
    OP_DICT_NEW,
    OP_MEMBER_GET, OP_MEMBER_GET_NO_POP,
    OP_MEMBER_SET, OP_DICT_LOAD, OP_ARRAY_LOAD,
    OP_INDEX_GET, OP_INDEX_SET,
    // exceptions
    OP_TRY, OP_RAISE, OP_EXFRAME_RET,
    // tail calls
    OP_RETCALL,
    // iterators
    OP_FOR_IN, OP_SWAP,
    // modules
    OP_USE,
}

#[repr(C)]
pub struct Vm {
    pub ip          : u32, // current instruction pointer
    pub localenv    : *mut Env,
    // pointer to current stack frame
    pub localenv_bp : *mut Env, // rust owns this, so drop it pls
    // pointer to start of pool of stack frames
    pub globalenv  : *mut CHashMap,
    // global environment, all unscoped variables/variables
    // starting with '$' should be stored here
    pub exframes   : CArray<ExFrame>, // exception frame
    pub code       : CArray<VmOpcode>, // where all the code is
    pub stack      : CArray<NativeValue>, // stack

    // prototype types for primitive values
    pub dstr       : *mut Record,
    pub dint       : *mut Record,
    pub dfloat     : *mut Record,
    pub darray     : *mut Record,
    pub drec       : *mut Record,

    pub error      : VmError,
    // whether the interpreter raised an unhandled error
    pub error_expected: u32,

    // for handling exceptions inside of interpreted functions called by native functions
    pub exframe_fallthrough: *const ExFrame,
    pub native_call_depth: usize,

    // rust-specific fields
    pub compiler   : Option<*mut Compiler>,
    // store compiler here for import modules
    // TODO: rethink the design and use RC
}

#[link(name="hana", kind="static")]
#[allow(improper_ctypes)]
extern "C" {
    fn vm_execute(vm: *mut Vm);
    fn vm_print_stack(vm: *const Vm);
    fn vm_call(vm: *mut Vm, fun: NativeValue, args: CArray<NativeValue>)
        -> NativeValue;

    fn vm_code_push8  (vm: *mut Vm, n : u8);
    fn vm_code_pushstr(vm: *mut Vm, s : *const libc::c_char);
    fn vm_code_pushf64(vm: *mut Vm, n : f64);
    fn vm_code_fill(vm: *mut Vm, pos : u32, len : u32);
    fn vm_code_fill16(vm: *mut Vm, pos : u32, len : u16);
}

impl Vm {

    #[cfg_attr(tarpaulin, skip)]
    pub fn new() -> Vm {
        Vm{
            ip: 0,
            localenv: null_mut(),
            localenv_bp: {
                use std::alloc::{alloc, Layout};
                use std::mem::size_of;
                let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
                unsafe { alloc(layout.unwrap()) as *mut Env }
            },
            globalenv: unsafe { Box::into_raw(Box::new(CHashMap::new())) },
            exframes: CArray::new(),
            code: CArray::new(),
            stack: CArray::new(),
            dstr: null_mut(),
            dint: null_mut(),
            dfloat: null_mut(),
            darray: null_mut(),
            drec: null_mut(),
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            exframe_fallthrough: null_mut(),
            native_call_depth: 0,
            compiler: None
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn print_stack(&self) {
        unsafe { vm_print_stack(self); }
    }

    pub fn execute(&mut self) {
        unsafe { vm_execute(self); }
    }

    // pushes
    pub fn cpush8(&mut self, n : u8) { unsafe { vm_code_push8(self, n); } }
    pub fn cpush16(&mut self, n : u16) {
        for byte in &n.to_be_bytes() {
            self.cpush8(*byte);
        }
    }
    pub fn cpush32(&mut self, n : u32) {
        for byte in &n.to_be_bytes() {
            self.cpush8(*byte);
        }
    }
    pub fn cpush64(&mut self, n : u64) {
        for byte in &n.to_be_bytes() {
            self.cpush8(*byte);
        }
    }
    pub fn cpushf64(&mut self, n : f64) { unsafe { vm_code_pushf64(self, n); } }
    pub fn cpushs<T : Into<Vec<u8>>>(&mut self, s : T) {
        let cstr = CString::new(s).expect("can't turn to cstring");
        unsafe { vm_code_pushstr(self, cstr.as_ptr()); }
    }

    #[cfg_attr(tarpaulin, skip)]
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
                for val in (*env).slots.as_mut_slice().iter_mut() {
                    (*val).mark();
                }
                env = env.add(1);
            }
            env = self.localenv;
            for val in (*env).slots.as_mut_slice().iter_mut() {
                (*val).mark();
            }
        } }
    }

    // call stack
    pub fn enter_env(&mut self, fun: &'static mut Function) {
        if self.localenv.is_null() {
            self.localenv = self.localenv_bp;
        } else {
            if unsafe {
                self.localenv.offset_from(self.localenv_bp) > (CALL_STACK_SIZE as isize)
            } { panic!("maximum stack depth exceeded"); }
            self.localenv = unsafe{ self.localenv.add(1) };
        }
        unsafe { std::ptr::write(self.localenv, Env::new(self.ip, fun.get_bound_ptr(), fun.nargs)); }
        self.ip = fun.ip;
    }

    pub fn enter_env_tail(&mut self, fun: &'static mut Function) {
        let env = unsafe{ &mut *self.localenv };
        env.nargs = fun.nargs;
        env.lexical_parent = unsafe{ fun.get_bound_ptr() };
        self.ip = fun.ip;
    }

    pub fn leave_env(&mut self) {
        // we don't check for env leaving
        // this must be non-null
        unsafe {
            self.ip = (*self.localenv).retip;
            if self.localenv == self.localenv_bp {
                std::ptr::drop_in_place(self.localenv);
                self.localenv = null_mut();
            } else {
                std::ptr::drop_in_place(self.localenv);
                self.localenv = self.localenv.sub(1);
            }
        }
    }

    // exceptions
    pub fn enter_exframe(&mut self) -> &mut ExFrame {
        self.exframes.push(ExFrame::new(self.localenv, self.stack.len()-1, self.native_call_depth));
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
                if exframe.unwind_native_call_depth != self.native_call_depth {
                    self.exframe_fallthrough = exframe;
                }
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
    pub fn call(&mut self, fun: NativeValue, args: CArray<NativeValue>) -> Option<NativeValue> {
        let val = unsafe{ vm_call(self, fun, args) };
        if self.code[self.ip as usize] == VmOpcode::OP_HALT ||
           self.exframe_fallthrough != null_mut() ||
           self.error != VmError::ERROR_NO_ERROR {
            None }
        else { Some(val) }
    }

    // execution context for eval
    pub fn new_exec_ctx(&mut self) -> ManuallyDrop<Vm> {
        // save current ctx
        let current_ctx = Vm{
            ip: self.ip,
            localenv: self.localenv,
            localenv_bp: self.localenv_bp,
            globalenv: null_mut(), // shared
            exframes: self.exframes.deref(),
            code: CArray::new_nil(), // shared
            stack: self.stack.deref(),
            // types don't need to be saved:
            dstr: null_mut(),
            dint: null_mut(),
            dfloat: null_mut(),
            darray: null_mut(),
            drec: null_mut(),
            // shared
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            exframe_fallthrough: null_mut(),
            native_call_depth: 0,
            compiler: None,
        };
        // create new ctx
        self.ip = 0;
        self.localenv = null_mut();
        self.localenv_bp = {
            use std::alloc::{alloc_zeroed, Layout};
            use std::mem::size_of;
            let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
            unsafe { alloc_zeroed(layout.unwrap()) as *mut Env }
        };
        self.exframes = CArray::new_nil();
        self.stack = CArray::reserve(2);
        ManuallyDrop::new(current_ctx)
    }

    pub fn restore_exec_ctx(&mut self, mut ctx: ManuallyDrop<Vm>) {
        self.ip = ctx.ip;
        self.localenv = ctx.localenv;
        self.localenv_bp = ctx.localenv_bp;
        self.exframes = ctx.exframes.deref();
        self.stack = ctx.stack.deref();
    }

    // imports
    pub fn load_module(&mut self, path: &String) {
        // loads module, jumps to the module then jump back to OP_USE
        use crate::ast;
        use std::io::Read;

        let mut c = unsafe{ &mut *self.compiler.unwrap() };

        let pathobj =
            if path.starts_with("./") {
                let last_path = c.files.last().unwrap();
                let curpath = Path::new(&last_path);
                eprintln!("{:?}", curpath);
                let mut pbuf = if let Some(parent) = curpath.parent() {
                    parent.join(Path::new(path))
                } else {
                    Path::new(path).to_path_buf()
                };
                if pbuf.extension().is_none() {
                    pbuf.set_extension("hana"); }
                pbuf
            } else if path.starts_with("/") {
                let mut pbuf = Path::new(path).to_path_buf();
                if pbuf.extension().is_none() {
                    pbuf.set_extension("hana"); }
                pbuf
            } else {
                use std::env;
                match env::var_os("HANA_PATH") {
                    Some(parent) =>
                        env::split_paths(&parent)
                            .map(|x| {
                                let mut pbuf = Path::new(&x)
                                    .join(path).to_path_buf();
                                if pbuf.extension().is_none() {
                                    pbuf.set_extension("hana"); }
                                pbuf
                            })
                            .find(|x| x.as_path().is_file())
                            .unwrap(),
                    None => panic!("HANA_PATH not set!")
                }
            };

        if c.modules_loaded.contains(&pathobj) {
            return;
        } else {
            c.modules_loaded.insert(pathobj.clone());
        }

        if let Ok(mut file) = std::fs::File::open(pathobj) {
            let mut s = String::new();
            file.read_to_string(&mut s);
            let prog = ast::grammar::start(&s).unwrap();
            c.files.push(path.clone());
            c.sources.push(s);

            let importer_ip = self.ip;
            let imported_ip = self.code.len();
            for stmt in prog {
                stmt.emit(&mut c);
            }
            self.code.push(VmOpcode::OP_JMP_LONG);
            self.cpush32(importer_ip);
            self.ip = imported_ip as u32;
        } else {
            return;
        }
    }
}

impl std::ops::Drop for Vm {
    fn drop(&mut self) {
        unsafe {
            use std::alloc::{dealloc, Layout};
            use std::mem;

            // stack frames
            if !self.localenv.is_null() {
                let mut env = self.localenv_bp;
                while env != self.localenv {
                    std::ptr::drop_in_place(env);
                    env = env.add(1);
                }
                std::ptr::drop_in_place(self.localenv);
            }
            let layout = Layout::from_size_align(mem::size_of::<Env>() * CALL_STACK_SIZE, 4);
            dealloc(self.localenv_bp as *mut u8, layout.unwrap());

            // exception frames
            self.exframes.drop();

            // primitive objects
            unpin(self.dstr as *mut libc::c_void);
            unpin(self.dint as *mut libc::c_void);
            unpin(self.dfloat as *mut libc::c_void);
            unpin(self.darray as *mut libc::c_void);
            unpin(self.drec as *mut libc::c_void);

            // other
            if !self.globalenv.is_null() {
                Box::from_raw(self.globalenv);
            }
            self.code.drop();
            self.stack.drop();

        }
    }
}
