//! Provides an interface for the virtual machine

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::ptr::null_mut;
use std::rc::Rc;

extern crate libc;

use super::carray::CArray;
use super::chmap::CHashMap;
use super::cnativeval::NativeValue;
use super::env::Env;
use super::exframe::ExFrame;
use super::function::Function;
use super::gc::*;
use super::record::Record;

use super::vmerror::VmError;
use crate::compiler::{Compiler, ModulesInfo};
use crate::hanayo::HanayoCtx;

extern crate num_derive;
use num_traits::cast::FromPrimitive;

const CALL_STACK_SIZE: usize = 512;

//
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, FromPrimitive, ToPrimitive)]
#[allow(non_camel_case_types)]
pub enum VmOpcode {
    OP_HALT,
    // stack manip
    OP_PUSH8,
    OP_PUSH16,
    OP_PUSH32,
    OP_PUSH64,
    OP_PUSH_NIL,
    OP_PUSHSTR,
    OP_PUSHF64,
    OP_POP,
    // arith
    OP_ADD,
    OP_SUB,
    OP_MUL,
    OP_DIV,
    OP_MOD,
    OP_IADD,
    OP_IMUL,
    // unary
    OP_NEGATE,
    OP_NOT,
    // comparison
    OP_LT,
    OP_LEQ,
    OP_GT,
    OP_GEQ,
    OP_EQ,
    OP_NEQ,
    // matching
    OP_OF, /* type matching */
    // variables
    OP_ENV_NEW,
    OP_SET_LOCAL,
    OP_SET_LOCAL_FUNCTION_DEF,
    OP_GET_LOCAL,
    OP_GET_LOCAL_UP,
    OP_SET_GLOBAL,
    OP_GET_GLOBAL,
    OP_DEF_FUNCTION_PUSH,
    // flow control
    OP_JMP,
    OP_JMP_LONG,
    OP_JCOND,
    OP_JNCOND,
    OP_CALL,
    OP_RET,
    OP_JCOND_NO_POP,
    OP_JNCOND_NO_POP,
    // dictionary
    OP_DICT_NEW,
    OP_MEMBER_GET,
    OP_MEMBER_GET_NO_POP,
    OP_MEMBER_SET,
    OP_DICT_LOAD,
    OP_ARRAY_LOAD,
    OP_INDEX_GET,
    OP_INDEX_GET_NO_POP,
    OP_INDEX_SET,
    // exceptions
    OP_TRY,
    OP_RAISE,
    OP_EXFRAME_RET,
    // tail calls
    OP_RETCALL,
    // iterators
    OP_FOR_IN,
    OP_SWAP,
    // modules
    OP_USE,
}

#[repr(C)]
pub struct Vm {
    ip: u32, // current instruction pointer
    localenv: *mut Env,
    // pointer to current stack frame
    localenv_bp: *mut Env, // rust owns this, so drop it pls
    // pointer to start of pool of stack frames
    globalenv: *mut CHashMap,
    // global environment, all unscoped variables/variables
    // starting with '$' should also be stored here without '$'
    exframes: CArray<ExFrame>,      // exception frame
    pub code: CArray<u8>,           // where all the code is
    pub stack: CArray<NativeValue>, // stack

    // prototype types for primitive values
    pub(crate) dstr: Gc<Record>,
    pub(crate) dint: Gc<Record>,
    pub(crate) dfloat: Gc<Record>,
    pub(crate) darray: Gc<Record>,
    pub(crate) drec: Gc<Record>,

    pub error: VmError,
    pub error_expected: u32,

    // for handling exceptions inside of interpreted functions called by native functions
    exframe_fallthrough: *const ExFrame,
    native_call_depth: usize,

    // rust-specific fields
    pub modules_info: Option<Rc<RefCell<ModulesInfo>>>,
    pub(crate) stdlib: Option<HanayoCtx>,
    gc_manager: Option<RefCell<GcManager>>,
}

#[link(name = "hana", kind = "static")]
#[allow(improper_ctypes)]
extern "C" {
    fn vm_execute(vm: *mut Vm);
    fn vm_print_stack(vm: *const Vm);
    fn vm_call(vm: *mut Vm, fun: NativeValue, args: *const CArray<NativeValue>) -> NativeValue;
}

impl Vm {
    #[cfg_attr(tarpaulin, skip)]
    pub fn new(code: CArray<u8>, modules_info: Option<Rc<RefCell<ModulesInfo>>>) -> Vm {
        Vm {
            ip: 0,
            localenv: null_mut(),
            localenv_bp: {
                use std::alloc::{alloc, Layout};
                use std::mem::size_of;
                let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
                unsafe { alloc(layout.unwrap()) as *mut Env }
            },
            globalenv: Box::into_raw(Box::new(CHashMap::new())),
            exframes: CArray::new(),
            code,
            stack: CArray::new(),
            dstr: Gc::new_nil(),
            dint: Gc::new_nil(),
            dfloat: Gc::new_nil(),
            darray: Gc::new_nil(),
            drec: Gc::new_nil(),
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            exframe_fallthrough: null_mut(),
            native_call_depth: 0,
            modules_info,
            stdlib: None,
            gc_manager: Some(RefCell::new(GcManager::new())),
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn print_stack(&self) {
        // TODO: move vm_print_stack here and expose function through C ffi
        unsafe {
            vm_print_stack(self);
        }
    }

    pub fn execute(&mut self) {
        unsafe {
            vm_execute(self);
        }
    }

    // globals
    pub fn global(&self) -> &CHashMap {
        if self.globalenv.is_null() {
            panic!("accessing nil ptr");
        }
        unsafe { &*self.globalenv }
    }

    pub fn mut_global(&self) -> &mut CHashMap {
        if self.globalenv.is_null() {
            panic!("accessing nil ptr");
        }
        unsafe { &mut *self.globalenv }
    }

    // gc
    pub fn malloc<T: Sized + GcTraceable>(&self, val: T) -> Gc<T> {
        self.gc_manager
            .as_ref()
            .unwrap()
            .borrow_mut()
            .malloc(self, val)
    }

    pub fn gc_disable(&self) {
        self.gc_manager.as_ref().unwrap().borrow_mut().disable()
    }
    pub fn gc_enable(&self) {
        self.gc_manager.as_ref().unwrap().borrow_mut().enable()
    }

    pub unsafe fn mark(&self) {
        // globalenv
        let globalenv = &*self.globalenv;
        for (_, val) in globalenv.iter() {
            val.trace();
        }
        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            val.trace();
        }
        // call stack
        if !self.localenv.is_null() {
            let mut env = self.localenv_bp;
            while env != self.localenv {
                for val in (*env).slots.as_mut_slice().iter_mut() {
                    (*val).trace();
                }
                env = env.add(1);
            }
            env = self.localenv;
            for val in (*env).slots.as_mut_slice().iter_mut() {
                (*val).trace();
            }
        }
    }

    // call stack
    pub unsafe fn enter_env(&mut self, fun: &'static Function) {
        if self.localenv.is_null() {
            self.localenv = self.localenv_bp;
        } else if self.localenv.offset_from(self.localenv_bp) > (CALL_STACK_SIZE as isize) {
            panic!("maximum stack depth exceeded");
        } else {
            self.localenv = self.localenv.add(1);
        }
        std::ptr::write(
            self.localenv,
            Env::new(self.ip, fun.get_bound_ptr(), fun.nargs),
        );
        self.ip = fun.ip;
    }

    pub unsafe fn enter_env_tail(&mut self, fun: &'static Function) {
        let env = &mut *self.localenv;
        env.nargs = fun.nargs;
        env.lexical_parent = fun.get_bound_ptr();
        self.ip = fun.ip;
    }

    pub unsafe fn leave_env(&mut self) {
        // we don't check for env leaving
        // this must be non-null
        self.ip = (*self.localenv).retip;
        if self.localenv == self.localenv_bp {
            std::ptr::drop_in_place(self.localenv);
            self.localenv = null_mut();
        } else {
            std::ptr::drop_in_place(self.localenv);
            self.localenv = self.localenv.sub(1);
        }
    }

    // accessors
    pub fn localenv(&self) -> *mut Env {
        self.localenv
    }
    #[cfg_attr(tarpaulin, skip)]
    pub fn localenv_is_null(&self) -> bool {
        self.localenv.is_null()
    }
    #[cfg_attr(tarpaulin, skip)]
    pub fn localenv_bp(&self) -> *mut Env {
        self.localenv_bp
    }
    #[cfg_attr(tarpaulin, skip)]
    pub fn localenv_to_vec(&self) -> Vec<Env> {
        // used for error handling and such
        if self.localenv.is_null() {
            return Vec::new();
        }
        let mut env = self.localenv;
        let mut vec = Vec::new();
        while env != unsafe { self.localenv_bp.sub(1) } {
            vec.push(unsafe { &*env }.clone());
            env = unsafe { env.sub(1) };
        }
        vec
    }

    // exceptions
    pub fn enter_exframe(&mut self) -> &mut ExFrame {
        self.exframes.push(ExFrame::new(
            self.localenv,
            self.stack.len() - 1,
            self.native_call_depth,
        ));
        self.exframes.top_mut()
    }
    pub fn leave_exframe(&mut self) {
        self.exframes.pop();
    }
    pub fn raise(&mut self) -> bool {
        if self.exframes.len() == 0 {
            return false;
        }
        let val = self.stack.top().unwrap();
        let mut i = self.exframes.len();
        while i != 0 {
            let exframe = &self.exframes[i - 1];
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
    pub fn call(&mut self, fun: NativeValue, args: &CArray<NativeValue>) -> Option<NativeValue> {
        let val = unsafe { vm_call(self, fun, args) };
        if let Some(opcode) = VmOpcode::from_u8(self.code[self.ip as usize]) {
            if opcode == VmOpcode::OP_HALT {
                return None;
            }
        }
        if self.exframe_fallthrough != null_mut() || self.error != VmError::ERROR_NO_ERROR {
            None
        } else {
            Some(val)
        }
    }

    // execution context for eval
    pub unsafe fn new_exec_ctx(&mut self) -> ManuallyDrop<Vm> {
        // save current ctx
        let current_ctx = Vm {
            ip: self.ip,
            localenv: self.localenv,
            localenv_bp: self.localenv_bp,
            globalenv: null_mut(), // shared
            exframes: self.exframes.deref(),
            code: CArray::new_nil(), // shared
            stack: self.stack.deref(),
            // types don't need to be saved:
            dstr: Gc::new_nil(),
            dint: Gc::new_nil(),
            dfloat: Gc::new_nil(),
            darray: Gc::new_nil(),
            drec: Gc::new_nil(),
            // shared
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            exframe_fallthrough: self.exframe_fallthrough,
            native_call_depth: self.native_call_depth,
            modules_info: None,
            stdlib: None,
            gc_manager: None,
        };
        // create new ctx
        self.ip = 0;
        self.localenv = null_mut();
        self.localenv_bp = {
            use std::alloc::{alloc_zeroed, Layout};
            use std::mem::size_of;
            let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
            alloc_zeroed(layout.unwrap()) as *mut Env
        };
        self.exframes = CArray::new_nil();
        self.stack = CArray::reserve(2);
        ManuallyDrop::new(current_ctx)
    }

    pub unsafe fn restore_exec_ctx(&mut self, ctx: ManuallyDrop<Vm>) {
        let mut ctx = ManuallyDrop::into_inner(ctx);

        // drop old
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

        // fill in
        self.ip = ctx.ip;
        self.localenv = ctx.localenv;
        self.localenv_bp = ctx.localenv_bp;
        self.exframes = ctx.exframes.deref();
        self.exframe_fallthrough = ctx.exframe_fallthrough;
        self.native_call_depth = ctx.native_call_depth;
        self.stack = ctx.stack.deref();

        // prevent double freeing:
        ctx.localenv = null_mut();
        ctx.localenv_bp = null_mut();
    }

    // instruction pointer
    pub fn ip(&self) -> u32 {
        self.ip
    }
    pub fn jmp(&mut self, ip: u32) {
        assert!(ip < self.code.len() as u32);
        self.ip = ip;
    }

    // imports
    pub fn load_module(&mut self, path: &str) {
        // loads module, jumps to the module then jump back to OP_USE
        use crate::ast;
        use std::io::Read;

        let rc = self.modules_info.clone().unwrap();
        let mut c = rc.borrow_mut();

        let pathobj = if path.starts_with("./") {
            let last_path = c.files.last().unwrap();
            let curpath = Path::new(&last_path);
            let mut pathobj = if let Some(parent) = curpath.parent() {
                parent.join(Path::new(path))
            } else {
                Path::new(path).to_path_buf()
            };
            if !pathobj.as_path().is_file() && pathobj.extension().is_none() {
                pathobj.set_extension("hana");
            }
            pathobj
        } else if path.starts_with('/') {
            let mut pathobj = Path::new(path).to_path_buf();
            if !pathobj.as_path().is_file() && pathobj.extension().is_none() {
                pathobj.set_extension("hana");
            }
            pathobj
        } else {
            use std::env;
            match env::var_os("HANA_PATH") {
                Some(parent) => env::split_paths(&parent)
                    .map(|x| {
                        let mut pathobj = Path::new(&x).join(path).to_path_buf();
                        if pathobj.extension().is_none() {
                            pathobj.set_extension("hana");
                        }
                        pathobj
                    })
                    .find(|x| x.as_path().is_file())
                    .unwrap(),
                None => panic!("HANA_PATH not set!"),
            }
        };

        if c.modules_loaded.contains(&pathobj) {
            return;
        } else {
            c.modules_loaded.insert(pathobj.clone());
        }

        if let Ok(mut file) = std::fs::File::open(pathobj) {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            let prog = ast::grammar::start(&s).unwrap();
            c.files.push(path.to_string());
            c.sources.push(s);

            let importer_ip = self.ip;
            let imported_ip = self.code.len();
            {
                let mut c = Compiler::new_append(unsafe { self.code.deref() });
                for stmt in prog {
                    stmt.emit(&mut c);
                }
                c.cpushop(VmOpcode::OP_JMP_LONG);
                c.cpush32(importer_ip);
                self.code = c.into_code();
            }
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

            // other
            if !self.globalenv.is_null() {
                Box::from_raw(self.globalenv);
            }
        }
    }
}
