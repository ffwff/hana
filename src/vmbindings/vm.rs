//! Provides an interface for the virtual machine

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::ptr::{null_mut, NonNull};
use std::rc::Rc;

extern crate libc;

use super::env::Env;
use super::exframe::ExFrame;
use super::function::Function;
use super::gc::*;
use super::hmap::HaruHashMap;
use super::interned_string_map::InternedStringMap;
use super::nativeval::{NativeValue, NativeValueType};
use super::record::Record;
use super::string::HaruString;
use super::value::Value;

use super::vmerror::VmError;
use crate::compiler::{Compiler, ModulesInfo};
use crate::hanayo::HanayoCtx;

extern crate num_derive;

const CALL_STACK_SIZE: usize = 512;

#[repr(transparent)]
struct ConstNonNull<T: Sized> {
    pointer: std::num::NonZeroUsize,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Sized> ConstNonNull<T> {
    pub fn new(pointer: *const T) -> Option<Self> {
        if !pointer.is_null() {
            unsafe {
                Some(ConstNonNull {
                    pointer: std::mem::transmute(pointer),
                    phantom: std::marker::PhantomData,
                })
            }
        } else {
            None
        }
    }
}

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
    OP_PUSHSTR_INTERNED,
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
    // bitwise
    OP_BITWISE_AND,
    OP_BITWISE_OR,
    OP_BITWISE_XOR,
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
    // pointer to current stack frame
    localenv: Option<NonNull<Env>>,
    // pointer to start of pool of stack frames
    localenv_bp: *mut Env,
    // global environment, all unscoped variables/variables
    // starting with '$' should also be stored here without '$'
    globalenv: Option<Box<HaruHashMap>>,
    exframes: Option<Vec<ExFrame>>, // exception frame
    pub code: Option<Vec<u8>>,      // where all the code is
    pub stack: Vec<NativeValue>,    // stack

    // prototype types for primitive values
    pub(crate) dstr: Option<Gc<Record>>,
    pub(crate) dint: Option<Gc<Record>>,
    pub(crate) dfloat: Option<Gc<Record>>,
    pub(crate) darray: Option<Gc<Record>>,
    pub(crate) drec: Option<Gc<Record>>,

    pub error: VmError,
    pub error_expected: u32,

    // for handling exceptions inside of interpreted functions called by native functions
    exframe_fallthrough: Option<ConstNonNull<ExFrame>>,
    native_call_depth: usize,

    // rust-specific fields
    pub interned_strings: Option<InternedStringMap>,
    pub modules_info: Option<Rc<RefCell<ModulesInfo>>>,
    pub(crate) stdlib: Option<HanayoCtx>,
    gc_manager: Option<RefCell<GcManager>>,
}

#[link(name = "hana", kind = "static")]
#[allow(improper_ctypes)]
extern "C" {
    fn vm_execute(vm: *mut Vm);
    fn vm_print_stack(vm: *const Vm);
    fn vm_call(vm: *mut Vm, fun: NativeValue, args: *const Vec<NativeValue>) -> NativeValue;
}

impl Vm {
    #[cfg_attr(tarpaulin, skip)]
    pub fn new(
        code: Option<Vec<u8>>, modules_info: Option<Rc<RefCell<ModulesInfo>>>,
        interned_strings: Option<InternedStringMap>,
    ) -> Vm {
        Vm {
            ip: 0,
            localenv: None,
            localenv_bp: {
                use std::alloc::{alloc, Layout};
                use std::mem::size_of;
                let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
                unsafe { alloc(layout.unwrap()) as *mut Env }
            },
            globalenv: Some(Box::new(HaruHashMap::new())),
            exframes: Some(Vec::with_capacity(2)),
            code,
            stack: Vec::with_capacity(2),
            dstr: None,
            dint: None,
            dfloat: None,
            darray: None,
            drec: None,
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            exframe_fallthrough: None,
            native_call_depth: 0,
            interned_strings,
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
        if self.code.is_none() {
            panic!("calling with nil code");
        }
        unsafe {
            vm_execute(self);
        }
    }

    // interned string
    pub unsafe fn get_interned_string(&self, n: u16) -> HaruString {
        HaruString::new_cow(self.interned_strings.as_ref().unwrap().get_unchecked(n).clone())
    }

    // globals
    pub fn global(&self) -> &HaruHashMap {
        use std::borrow::Borrow;
        self.globalenv.as_ref().unwrap().borrow()
    }

    pub fn mut_global(&mut self) -> &mut HaruHashMap {
        use std::borrow::BorrowMut;
        self.globalenv.as_mut().unwrap().borrow_mut()
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

    pub unsafe fn stack_push_gray(&mut self, val: Value) {
        let w = val.wrap();
        if let Some(ptr) = w.as_gc_pointer() {
            self.gc_manager
                .as_ref()
                .unwrap()
                .borrow_mut()
                .push_gray_body(ptr);
        }
        self.stack.push(w);
    }

    // call stack
    pub unsafe fn enter_env(&mut self, fun: &'static Function) {
        if self.localenv.is_none() {
            self.localenv = NonNull::new(self.localenv_bp);
        } else {
            let localenv = self.localenv.unwrap().as_ptr();
            if localenv.offset_from(self.localenv_bp) > (CALL_STACK_SIZE as isize) {
                panic!("maximum stack depth exceeded");
            } else {
                self.localenv = NonNull::new(localenv.add(1));
            }
        }
        std::ptr::write(
            self.localenv.unwrap().as_ptr(),
            Env::new(self.ip, fun.get_bound_ptr(), fun.nargs),
        );
        self.ip = fun.ip;
    }

    pub unsafe fn enter_env_tail(&mut self, fun: &'static Function) {
        let env = self.localenv.as_mut().unwrap().as_mut();
        env.nargs = fun.nargs;
        env.lexical_parent = fun.get_bound_ptr();
        self.ip = fun.ip;
    }

    pub unsafe fn leave_env(&mut self) {
        if let Some(localenv) = self.localenv {
            let localenv = localenv.as_ptr();
            self.ip = (*localenv).retip;
            if localenv == self.localenv_bp {
                std::ptr::drop_in_place(localenv);
                self.localenv = None;
            } else {
                std::ptr::drop_in_place(localenv);
                self.localenv = NonNull::new(localenv.sub(1));
            }
        }
    }

    // accessors
    pub fn localenv(&self) -> Option<NonNull<Env>> {
        self.localenv.clone()
    }
    /// Converts call stack into vector of stack frames.
    ///
    /// This is used for error handling and such.
    #[cfg_attr(tarpaulin, skip)]
    pub fn localenv_to_vec(&self) -> Vec<Env> {
        if self.localenv.is_none() {
            return Vec::new();
        }
        let mut env = self.localenv.unwrap().as_ptr();
        let mut vec = Vec::new();
        while env != unsafe { self.localenv_bp.sub(1) } {
            vec.push(unsafe { &*env }.clone());
            env = unsafe { env.sub(1) };
        }
        vec
    }

    // exceptions
    fn exframes(&self) -> &Vec<ExFrame> {
        self.exframes.as_ref().unwrap()
    }
    fn mut_exframes(&mut self) -> &mut Vec<ExFrame> {
        self.exframes.as_mut().unwrap()
    }
    pub fn enter_exframe(&mut self) -> &mut ExFrame {
        let localenv = self.localenv.clone();
        let len = self.stack.len() - 1;
        let native_call_depth = self.native_call_depth;
        self.mut_exframes()
            .push(ExFrame::new(localenv, len, native_call_depth));
        self.mut_exframes().last_mut().unwrap()
    }
    pub fn leave_exframe(&mut self) {
        self.mut_exframes().pop();
    }
    pub fn raise(&mut self) -> bool {
        if self.exframes().len() == 0 {
            return false;
        }
        let val = unsafe { self.stack.last().unwrap().unwrap() };
        for exframe in self.exframes.as_ref().unwrap().iter() {
            if let Some(handler) = exframe.get_handler(self, &val) {
                self.ip = handler.ip;
                if handler.nargs == 0 {
                    self.stack.pop();
                }
                if exframe.unwind_native_call_depth != self.native_call_depth {
                    self.exframe_fallthrough = ConstNonNull::new(exframe);
                }
                return true;
            }
        }
        false
    }

    // functions
    pub fn call(&mut self, fun: NativeValue, args: &Vec<NativeValue>) -> Option<NativeValue> {
        let val = unsafe { vm_call(self, fun, args) };
        if val.r#type == NativeValueType::TYPE_INTERPRETER_ERROR {
            None
        } else {
            Some(val)
        }
    }

    // execution context for eval
    pub fn new_exec_ctx(&mut self) -> ManuallyDrop<Vm> {
        // prevent context's local variables from being freed
        unsafe {
            // stack
            let stack = &self.stack;
            for val in stack.iter() {
                val.ref_inc();
            }
            // call stack
            if let Some(localenv) = self.localenv {
                let mut env = self.localenv_bp;
                let localenv = localenv.as_ptr();
                while env != localenv {
                    for val in (*env).slots.iter() {
                        (*val).ref_inc();
                    }
                    env = env.add(1);
                }
                env = localenv;
                for val in (*env).slots.iter() {
                    (*val).ref_inc();
                }
            }
        }
        // save current ctx
        let current_ctx = Vm {
            ip: self.ip,
            localenv: self.localenv.take(),
            localenv_bp: self.localenv_bp,
            globalenv: None, // shared
            exframes: self.exframes.take(),
            code: None, // shared
            stack: std::mem::replace(&mut self.stack, Vec::with_capacity(2)),
            // types don't need to be saved:
            dstr: None,
            dint: None,
            dfloat: None,
            darray: None,
            drec: None,
            // shared
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            interned_strings: None,
            exframe_fallthrough: self.exframe_fallthrough.take(),
            native_call_depth: self.native_call_depth,
            modules_info: None,
            stdlib: None,
            gc_manager: None,
        };
        // create new ctx
        self.ip = 0;
        self.localenv_bp = unsafe {
            use std::alloc::{alloc_zeroed, Layout};
            use std::mem::size_of;
            let layout = Layout::from_size_align(size_of::<Env>() * CALL_STACK_SIZE, 4);
            alloc_zeroed(layout.unwrap()) as *mut Env
        };
        self.exframes = Some(Vec::new());
        ManuallyDrop::new(current_ctx)
    }

    pub fn restore_exec_ctx(&mut self, ctx: ManuallyDrop<Vm>) {
        let mut ctx: Vm = ManuallyDrop::into_inner(ctx);

        // drop old
        unsafe {
            use std::alloc::{dealloc, Layout};
            use std::mem;
            // stack frames
            if let Some(localenv) = self.localenv {
                let mut env = self.localenv_bp;
                while env != localenv.as_ptr() {
                    std::ptr::drop_in_place(env);
                    env = env.add(1);
                }
                std::ptr::drop_in_place(localenv.as_ptr());
            }
            let layout = Layout::from_size_align(mem::size_of::<Env>() * CALL_STACK_SIZE, 4);
            dealloc(self.localenv_bp as *mut u8, layout.unwrap());
        }

        // fill in
        self.ip = ctx.ip;
        self.localenv = ctx.localenv.take();
        self.localenv_bp = ctx.localenv_bp;
        self.exframes = ctx.exframes.take();
        self.exframe_fallthrough = ctx.exframe_fallthrough.take();
        self.native_call_depth = ctx.native_call_depth;
        self.stack = std::mem::replace(&mut ctx.stack, Vec::new());

        // release context's local variables
        unsafe {
            // stack
            let stack = &self.stack;
            for val in stack.iter() {
                val.ref_dec();
            }
            // call stack
            if let Some(localenv) = self.localenv {
                let mut env = self.localenv_bp;
                let localenv = localenv.as_ptr();
                while env != localenv {
                    for val in (*env).slots.iter() {
                        (*val).ref_dec();
                    }
                    env = env.add(1);
                }
                env = localenv;
                for val in (*env).slots.iter() {
                    (*val).ref_dec();
                }
            }
        }

        // prevent double freeing:
        ctx.localenv_bp = null_mut();
    }

    // instruction pointer
    pub fn ip(&self) -> u32 {
        self.ip
    }
    pub fn jmp(&mut self, ip: u32) {
        assert!(ip < self.code.as_ref().unwrap().len() as u32);
        self.ip = ip;
    }

    // imports
    pub fn load_module(&mut self, path: &str) {
        // loads module, jumps to the module then jump back to OP_USE
        use crate::ast;
        use std::io::Read;

        let rc = self.modules_info.clone().unwrap();

        let pathobj = if path.starts_with("./") {
            let c = rc.borrow_mut();
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

        if rc.borrow_mut().modules_loaded.contains(&pathobj) {
            return;
        } else {
            rc.borrow_mut().modules_loaded.insert(pathobj.clone());
        }

        if let Ok(mut file) = std::fs::File::open(pathobj) {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            let prog = ast::grammar::start(&s).unwrap();
            rc.borrow_mut().files.push(path.to_string());
            rc.borrow_mut().sources.push(s);

            let importer_ip = self.ip;
            let imported_ip = self.code.as_ref().unwrap().len();
            {
                let mut c = Compiler::new_append(
                    self.code.take().unwrap(),
                    rc,
                    self.interned_strings.take().unwrap(),
                );
                for stmt in prog {
                    stmt.emit(&mut c).unwrap();
                }
                c.cpushop(VmOpcode::OP_JMP_LONG);
                c.cpush32(importer_ip);
                self.interned_strings = c.interned_strings.take();
                self.code = Some(c.into_code());
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
            if let Some(localenv) = self.localenv {
                let mut env = self.localenv_bp;
                while env != localenv.as_ptr() {
                    std::ptr::drop_in_place(env);
                    env = env.add(1);
                }
                std::ptr::drop_in_place(localenv.as_ptr());
            }
            let layout = Layout::from_size_align(mem::size_of::<Env>() * CALL_STACK_SIZE, 4);
            dealloc(self.localenv_bp as *mut u8, layout.unwrap());
        }
    }
}

impl GcTraceable for Vm {

    unsafe fn trace(&self, vec: &mut Vec<*mut GcNode>) {
        for (_, val) in self.global().iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(vec, ptr);
            }
        }
        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(vec, ptr);
            }
        }
        // call stack
        if let Some(localenv) = self.localenv {
            let mut env = self.localenv_bp;
            let localenv = localenv.as_ptr();
            while env != localenv {
                for val in (*env).slots.iter() {
                    if let Some(ptr) = (*val).as_gc_pointer() {
                        push_gray_body(vec, ptr);
                    }
                }
                env = env.add(1);
            }
            env = localenv;
            for val in (*env).slots.iter() {
                if let Some(ptr) = (*val).as_gc_pointer() {
                    push_gray_body(vec, ptr);
                }
            }
        }
    }

}