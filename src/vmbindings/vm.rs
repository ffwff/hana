//! Provides an interface for the virtual machine

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::ptr::{null_mut, NonNull};
use std::rc::Rc;
use std::ffi::CStr;

extern crate libc;

use super::env::Env;
use super::exframe::ExFrame;
use super::function::Function;
use super::gc::*;
use super::hmap::HaruHashMap;
use super::interned_string_map::InternedStringMap;
use super::record::Record;
use super::string::HaruString;
use super::value::Value;

use super::vmerror::VmError;
use crate::compiler::{Compiler, ModulesInfo};
//use crate::hanayo::HanayoCtx;

extern crate num_derive;
use num_traits::FromPrimitive;

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

    pub unsafe fn as_ref(&self) -> &T {
        &*(std::mem::transmute::<_, *const T>(self.pointer))
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
    pub stack: Vec<Value>,    // stack

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
    interned_strings: InternedStringMap,
    pub modules_info: Option<Rc<RefCell<ModulesInfo>>>,
    //pub(crate) stdlib: Option<HanayoCtx>,
    stdlib: Option<u8>,
    gc_manager: Option<RefCell<GcManager>>,
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
            interned_strings: interned_strings.unwrap_or_else(|| InternedStringMap::new()),
            modules_info,
            stdlib: None,
            gc_manager: Some(RefCell::new(GcManager::new())),
        }
    }

    // helper
    #[cfg_attr(tarpaulin, skip)]
    pub unsafe fn print_stack(&self) {
        // TODO: move vm_print_stack here and expose function through C ffi
        eprint!("[");
        for val in &self.stack {
            eprint!("{:?} ", val);
        }
        eprintln!("]");
    }

    pub fn execute(&mut self) {
        let code = self.code.as_ref().unwrap();
        if self.globalenv.is_none() {
            panic!("globalenv must not be none");
        }

        // #region macros
        macro_rules! unreachable_unchecked {
            () => {
                unsafe{ std::hint::unreachable_unchecked() }
            };
        }
        macro_rules! global {
            () => {
                self.globalenv.as_mut().unwrap_or_else(|| unreachable_unchecked!())
            };
        }
        // #region stack
        macro_rules! pop {
            () => {
                {
                    let last = last!().clone();
                    unsafe {
                        self.stack.set_len(self.stack.len() - 1);
                    }
                    last
                }
            };
        }
        macro_rules! last {
            () => {
                self.stack.last().unwrap_or_else(|| unreachable_unchecked!())
            };
        }
        // #endregion
        // #region code
        macro_rules! consume_u8 {
            () => {
                {
                    let n = unsafe{ *code.get_unchecked(self.ip as usize) };
                    self.ip += 1;
                    n
                }
            };
        }
        macro_rules! consume_u16 {
            () => {
                {
                    let mut num: [u8; 2] = Default::default();
                    num.copy_from_slice(unsafe{ code.get_unchecked((self.ip as usize)..(self.ip as usize + 2)) });
                    self.ip += 2;
                    u16::from_be_bytes(num)
                }
            };
        }
        macro_rules! peek_i16 {
            () => {
                {
                    let mut num: [u8; 2] = Default::default();
                    num.copy_from_slice(unsafe{ code.get_unchecked((self.ip as usize)..(self.ip as usize + 2)) });
                    i16::from_be_bytes(num)
                }
            };
        }
        macro_rules! peek_u16 {
            () => {
                {
                    let mut num: [u8; 2] = Default::default();
                    num.copy_from_slice(unsafe{ code.get_unchecked((self.ip as usize)..(self.ip as usize + 2)) });
                    u16::from_be_bytes(num)
                }
            };
        }
        macro_rules! consume_string {
            () => {
                {
                    let s = unsafe{ CStr::from_ptr(std::mem::transmute(code.get_unchecked(self.ip as usize))) }.to_string_lossy().to_string();
                    self.ip += s.len() as u32 + 1;
                    s
                }
            };
        }
        // #endregion
        // #region opcode
        macro_rules! op_push_int {
            ($type:ty) => {
                {
                    self.ip += 1;
                    let n = std::mem::size_of::<$type>();
                    let mut num: [u8; std::mem::size_of::<$type>()] = Default::default();
                    num.copy_from_slice(unsafe{ code.get_unchecked((self.ip as usize)..(self.ip as usize + n)) });
                    self.stack.push(Value::Int(<$type>::from_be_bytes(num) as i64));
                    self.ip += n as u32;
                }
            };
        }
        macro_rules! op_binary {
            ($func:ident) => {
                {
                    self.ip += 1;
                    let right = pop!();
                    let left  = pop!();
                    match left.$func(right, &self) {
                        Ok(retval) => self.stack.push(retval),
                        Err(e) => {
                            self.error = e;
                            return;
                        }
                    }
                }
            };
        }
        macro_rules! op_binary_in_place {
            ($func:ident) => {
                {
                    self.ip += 1;
                    let right = pop!();
                    let left = pop!();
                    match left.$func(right, &self) {
                        Ok((in_place, retval)) => {
                            self.stack.push(retval);
                            if in_place {
                                let ip = consume_u8!();
                                self.ip += ip as u32;
                            } else {
                                self.ip += 1;
                            }
                        }
                        Err(e) => {
                            self.error = e;
                            return;
                        }
                    }
                }
            };
        }
        macro_rules! op_compare {
            ($func:ident) => {
                {
                    self.ip += 1;
                    let right = pop!();
                    let left  = pop!();
                    self.stack.push(Value::Int(left.$func(right) as i64));
                }
            };
        }
        macro_rules! call_native {
            ($func:expr, $nargs:expr) => {
                self.native_call_depth += 1;
                unsafe {
                    // TODO this is a hack to get around the borrow checker
                    // pls fix this or it makes ferris sad
                    ($func)(std::mem::transmute(&*self), $nargs);
                }
                self.native_call_depth -= 1;
                if let Some(exframe) = &self.exframe_fallthrough {
                    let exframe = unsafe{ exframe.as_ref() };
                    if exframe.unwind_native_call_depth != self.native_call_depth {
                        return;
                    }
                } else if self.error != VmError::ERROR_NO_ERROR {
                    return;
                }
            };
        }
        // #endregion
        // #endregion

        loop {
            match unsafe {
                VmOpcode::from_u8(*code.get_unchecked(self.ip as usize))
                    .map_or_else(
                        || unreachable_unchecked!(),
                        |op| {
                            //eprintln!("{:?}", op);
                            op
                        }
                    )
            } {
                VmOpcode::OP_HALT => {
                    return;
                }
                // #region stack manip
                VmOpcode::OP_POP => {
                    self.ip += 1;
                    pop!();
                }

                VmOpcode::OP_PUSH_NIL => {
                    self.ip += 1;
                    self.stack.push(Value::Nil);
                }

                // #region push uint family
                VmOpcode::OP_PUSH8  => op_push_int!(u8),
                VmOpcode::OP_PUSH16 => op_push_int!(u16),
                VmOpcode::OP_PUSH32 => op_push_int!(u32),
                VmOpcode::OP_PUSH64 => op_push_int!(u64),
                // #endregion

                VmOpcode::OP_PUSHF64 => {
                    self.ip += 1;
                    let n = std::mem::size_of::<f64>();
                    let mut num: [u8; std::mem::size_of::<f64>()] = Default::default();
                    num.copy_from_slice(unsafe{ code.get_unchecked((self.ip as usize)..(self.ip as usize + n)) });
                    self.stack.push(Value::Float(unsafe{ std::mem::transmute(num) }));
                    self.ip += n as u32;
                }

                // #region push str
                VmOpcode::OP_PUSHSTR => {
                    self.ip += 1;
                    let s = consume_string!();
                    self.stack.push(Value::Str(self.malloc(s.into())));
                }

                VmOpcode::OP_PUSHSTR_INTERNED => {
                    self.ip += 1;
                    let idx = consume_u16!();
                    let inst = unsafe { HaruString::new_cow(self.interned_strings.get_unchecked(idx).clone()) };
                    self.stack.push(Value::Str(self.malloc(inst)));
                }
                // #endregion

                // push arrays
                VmOpcode::OP_ARRAY_LOAD => {
                    self.ip += 1;
                    let len = pop!();
                    match len {
                        Value::Int(0) => self.stack.push(Value::Array(self.malloc(Vec::new()))),
                        Value::Int(len) => {
                            let mut len = len as usize;
                            let mut v = self.malloc(vec![Value::Nil; len]);
                            while len > 0 {
                                v.as_mut()[len - 1] = pop!();
                                len -= 1;
                            }
                            self.stack.push(Value::Array(v));
                        }
                        _ => unreachable_unchecked!()
                    }
                }
                // push dicts
                VmOpcode::OP_DICT_LOAD => {
                    self.ip += 1;
                    let len = pop!();
                    match len {
                        Value::Int(0) => self.stack.push(Value::Record(self.malloc(Record::new()))),
                        Value::Int(len) => {
                            let mut len = len as usize;
                            let mut v = self.malloc(Record::with_capacity(len));
                            while len > 0 {
                                let key = pop!();
                                let val = unsafe{ pop!() };
                                match key {
                                    Value::Str(key) => {
                                        let key = key.as_ref().clone();
                                        v.as_mut().insert(key, val.clone());
                                    }
                                    _ => panic!("must be string"),
                                }
                                len -= 1;
                            }
                            self.stack.push(Value::Record(v));
                        }
                        _ => unreachable_unchecked!()
                    }
                }

                // push functions
                VmOpcode::OP_DEF_FUNCTION_PUSH => {
                    self.ip += 1;
                    let nargs = consume_u16!();
                    let pos = peek_u16!();
                    unsafe {
                        self.stack.push(Value::Fn(self.malloc(Function::new(self.ip + 2, nargs, self.localenv))));
                    }
                    self.ip += pos as u32;
                }

                // #region binary ops
                VmOpcode::OP_ADD => op_binary!(add),
                VmOpcode::OP_SUB => op_binary!(sub),
                VmOpcode::OP_MUL => op_binary!(mul),
                VmOpcode::OP_DIV => op_binary!(div),
                // comparisons
                VmOpcode::OP_GT => op_compare!(gt),
                VmOpcode::OP_LT => op_compare!(lt),
                VmOpcode::OP_EQ => op_compare!(eq),
                VmOpcode::OP_GEQ => op_compare!(geq),
                VmOpcode::OP_LEQ => op_compare!(leq),
                VmOpcode::OP_NEQ => op_compare!(neq),
                // in place
                VmOpcode::OP_IADD => op_binary_in_place!(add_in_place),
                VmOpcode::OP_IMUL => op_binary_in_place!(mul_in_place),
                // #endregion

                // #region unary ops
                VmOpcode::OP_NEGATE => {
                    self.ip += 1;
                    let val = pop!();
                    match val.negate(&self) {
                        Ok(retval) => self.stack.push(retval),
                        Err(e) => {
                            self.error = e;
                            return;
                        }
                    }
                }
                VmOpcode::OP_NOT => {
                    self.ip += 1;
                    let val = pop!();
                    self.stack.push(Value::Int(!val.is_true() as i64));
                }
                // #endregion
                // #endregion

                // #region indexing
                op @ VmOpcode::OP_INDEX_GET
                | op @ VmOpcode::OP_INDEX_GET_NO_POP => {
                    self.ip += 1;
                    let index = pop!();
                    let indexor = if op == VmOpcode::OP_INDEX_GET {
                        pop!()
                    } else {
                        last!().clone()
                    };

                    match indexor {
                        Value::Array(array) => {
                            let idx = match index {
                                Value::Int(idx) => idx as usize,
                                _ => {
                                    self.error = VmError::ERROR_KEY_NON_INT;
                                    return;
                                }
                            };
                            if let Some(val) = array.as_ref().get(idx) {
                                self.stack.push(val.clone());
                            } else {
                                self.error = VmError::ERROR_UNBOUNDED_ACCESS;
                                return;
                            }
                        }
                        Value::Str(string) => {
                            use unicode_segmentation::UnicodeSegmentation;
                            let idx: usize = match index {
                                Value::Int(idx) => idx as usize,
                                _ => {
                                    self.error = VmError::ERROR_KEY_NON_INT;
                                    return;
                                }
                            };
                            if let Some(val) = string.as_ref().as_str().graphemes(true).skip(idx).next() {
                                self.stack.push(Value::Str(self.malloc(val.clone().into())));
                            } else {
                                self.error = VmError::ERROR_UNBOUNDED_ACCESS;
                                return;
                            }
                        }
                        Value::Record(rec) => {
                            let idx: String = match index {
                                Value::Str(s) => s.as_ref().as_str().to_string(),
                                _ => {
                                    self.error = VmError::ERROR_RECORD_KEY_NON_STRING;
                                    return;
                                }
                            };
                            if let Some(val) = rec.as_ref().get(&idx) {
                                self.stack.push(val.clone());
                            } else {
                                self.error = VmError::ERROR_UNKNOWN_KEY;
                                return;
                            }
                        }
                        _ => panic!("cant access")
                    }
                }

                VmOpcode::OP_INDEX_SET => {
                    self.ip += 1;
                    let index = pop!();
                    let indexor = pop!();
                    let val = last!().clone();

                    match indexor {
                        Value::Array(array) => {
                            let idx: usize = match index {
                                Value::Int(idx) => idx as usize,
                                _ => {
                                    self.error = VmError::ERROR_KEY_NON_INT;
                                    return;
                                }
                            };
                            if (0..array.as_ref().len()).contains(&idx) {
                                array.as_mut()[idx] = val;
                            } else {
                                self.error = VmError::ERROR_UNBOUNDED_ACCESS;
                                return;
                            }
                        }
                        Value::Record(rec) => {
                            let idx: HaruString = match index {
                                Value::Str(s) => s.as_ref().clone(),
                                _ => {
                                    self.error = VmError::ERROR_RECORD_KEY_NON_STRING;
                                    return;
                                }
                            };
                            rec.as_mut().insert(idx, val);
                        }
                        _ => panic!("cant access")
                    }
                }

                op @ VmOpcode::OP_MEMBER_GET
                | op @ VmOpcode::OP_MEMBER_GET_NO_POP => {
                    self.ip += 1;
                    let key = consume_string!();
                    let last = last!().clone();
                    let dict = match last {
                        Value::Record(d) => d,
                        Value::Str(_) =>   self.dstr.clone().unwrap(),
                        Value::Int(_) =>   self.dint.clone().unwrap(),
                        Value::Float(_) => self.dfloat.clone().unwrap(),
                        Value::Array(_) => self.darray.clone().unwrap(),
                        _ => panic!("nope")
                    };
                    if op == VmOpcode::OP_MEMBER_GET {
                        pop!();
                    }
                    self.stack.push(dict.as_ref().get(&key).unwrap().clone());
                }

                VmOpcode::OP_MEMBER_SET => {
                    self.ip += 1;
                    let key = consume_string!();
                    let last = pop!().clone();
                    let val = last!().clone();
                    let dict = match last {
                        Value::Record(d) => d,
                        Value::Str(_) =>   self.dstr.clone().unwrap(),
                        Value::Int(_) =>   self.dint.clone().unwrap(),
                        Value::Float(_) => self.dfloat.clone().unwrap(),
                        Value::Array(_) => self.darray.clone().unwrap(),
                        _ => panic!("nope")
                    };
                    dict.as_mut().insert(key, val);

                }
                // #endregion

                // #region control flow
                VmOpcode::OP_JMP => {
                    self.ip += 1;
                    let ip = peek_i16!();
                    self.ip = (self.ip as i32 + ip as i32) as u32;
                }
                VmOpcode::OP_JCOND => {
                    self.ip += 1;
                    let ip = peek_i16!();
                    let val = pop!();
                    if val.is_true() {
                        self.ip = (self.ip as i32 + ip as i32) as u32;
                    } else {
                        self.ip += 2;
                    }
                }
                VmOpcode::OP_JNCOND => {
                    self.ip += 1;
                    let ip = peek_i16!();
                    let val = pop!();
                    if !val.is_true() {
                        self.ip = (self.ip as i32 + ip as i32) as u32;
                    } else {
                        self.ip += 2;
                    }
                }
                VmOpcode::OP_JCOND_NO_POP => {
                    self.ip += 1;
                    let ip = peek_i16!();
                    let val = last!().clone();
                    if val.is_true() {
                        self.ip = (self.ip as i32 + ip as i32) as u32;
                    } else {
                        self.ip += 2;
                    }
                }
                VmOpcode::OP_JNCOND_NO_POP => {
                    self.ip += 1;
                    let ip = peek_i16!();
                    let val = last!().clone();
                    if !val.is_true() {
                        self.ip = (self.ip as i32 + ip as i32) as u32;
                    } else {
                        self.ip += 2;
                    }
                }
                // #endregion

                // #region calling
                VmOpcode::OP_CALL => {
                    self.ip += 1;
                    let nargs = consume_u16!();
                    let val = last!().clone();
                    match val {
                        Value::NativeFn(x) => {
                            pop!();
                            call_native!(x, nargs);
                        }
                        Value::Fn(x) => {
                            pop!();
                            if x.as_ref().nargs != nargs {
                                panic!("nop");
                            }
                            let func = x.as_ref();
                            unsafe {
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
                                    Env::new(self.ip, func.get_bound_ptr(), func.nargs),
                                );
                            }
                            self.ip = func.ip;
                        }
                        _ => panic!("uncallable")
                    }
                }

                VmOpcode::OP_ENV_NEW => {
                    self.ip += 1;
                    let nslots = consume_u16!();
                    unsafe {
                        let mut ptr = self.localenv().clone().unwrap();
                        let env = ptr.as_mut();
                        env.reserve(nslots);
                        for i in 0..env.nargs {
                            let val = pop!();
                            env.set(i, val.clone());
                        }
                    }
                }

                VmOpcode::OP_RET => {
                    let mut ptr = self.localenv().clone().unwrap();
                    unsafe {
                        let env = ptr.as_mut();
                        if env.retip == std::u32::MAX {
                            return;
                        }
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
                }
                // #endregion

                // #region variables

                // #region locals
                VmOpcode::OP_GET_LOCAL => {
                    self.ip += 1;
                    let slot = consume_u16!();
                    unsafe {
                        self.stack.push(self.localenv().unwrap().as_mut().get(slot).clone());
                    }
                }
                VmOpcode::OP_GET_LOCAL_UP => {
                    self.ip += 1;
                    let slot = consume_u16!();
                    let relascope = consume_u16!();
                    unsafe {
                        self.stack.push(self.localenv().unwrap().as_mut().get_up(relascope, slot).clone());
                    }
                }
                VmOpcode::OP_SET_LOCAL => {
                    self.ip += 1;
                    let slot = consume_u16!();
                    let val = last!();
                    unsafe {
                        self.localenv().unwrap().as_mut().set(slot, val.clone());
                    }
                }
                VmOpcode::OP_SET_LOCAL_FUNCTION_DEF => {
                    self.ip += 1;
                    let slot = consume_u16!();
                    let val = last!();
                    unsafe {
                        self.localenv().unwrap().as_mut().set(slot, val.clone());
                        match val {
                            Value::Fn(func) => {
                                func.as_mut().get_mut_bound().set(0, val.clone());
                            }
                            _ => unreachable_unchecked!()
                        }
                    }
                }
                // #endregion

                // #region globals
                VmOpcode::OP_GET_GLOBAL => {
                    self.ip += 1;
                    let var = consume_string!();
                    self.stack.push(global!().get(var.as_str()).unwrap().clone());
                }
                VmOpcode::OP_SET_GLOBAL => {
                    self.ip += 1;
                    let var = consume_string!();
                    global!().insert(var.into(), last!().clone());
                }
                // #endregion

                // #endregion

                op => unreachable_unchecked!()
            }
            //unsafe{ self.print_stack(); }
        }
    }

    // interned strings
    pub unsafe fn get_interned_string(&self, n: u16) -> HaruString {
        HaruString::new_cow(self.interned_strings.get_unchecked(n).clone())
    }

    // #region globals
    pub fn global(&self) -> &HaruHashMap {
        use std::borrow::Borrow;
        self.globalenv.as_ref().unwrap().borrow()
    }

    pub fn mut_global(&mut self) -> &mut HaruHashMap {
        use std::borrow::BorrowMut;
        self.globalenv.as_mut().unwrap().borrow_mut()
    }
    // #endregion

    // #region gc
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
        let w = val;
        if let Some(ptr) = w.as_gc_pointer() {
            self.gc_manager
                .as_ref()
                .unwrap()
                .borrow_mut()
                .push_gray_body(ptr);
        }
        self.stack.push(w);
    }

    pub unsafe fn gc_new_gray_node_stack(&self) -> Vec<*mut GcNode> {
        let mut vec = Vec::new();
        for (_, val) in self.global().iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(&mut vec, ptr);
            }
        }
        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(&mut vec, ptr);
            }
        }
        // call stack
        if let Some(localenv) = self.localenv {
            let mut env = self.localenv_bp;
            let localenv = localenv.as_ptr();
            while env != localenv {
                for val in (*env).slots.iter() {
                    if let Some(ptr) = (*val).as_gc_pointer() {
                        push_gray_body(&mut vec, ptr);
                    }
                }
                env = env.add(1);
            }
            env = localenv;
            for val in (*env).slots.iter() {
                if let Some(ptr) = (*val).as_gc_pointer() {
                    push_gray_body(&mut vec, ptr);
                }
            }
        }
        vec
    }
    // #endregion

    // #region call stack
    pub unsafe fn enter_env(&mut self, fun: &Function) {
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
    // #endregion

    // #region exceptions
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
        let val = unsafe { self.stack.last().unwrap() };
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
    // #endregion

    // #region execution context for eval
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
            interned_strings: std::mem::replace(
                &mut self.interned_strings,
                InternedStringMap::new(),
            ),
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
    // #endregion

    // #region instruction pointer
    pub fn ip(&self) -> u32 {
        self.ip
    }
    pub fn jmp(&mut self, ip: u32) {
        assert!(ip < self.code.as_ref().unwrap().len() as u32);
        self.ip = ip;
    }
    // #endregion

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
                    std::mem::replace(&mut self.interned_strings, InternedStringMap::new()),
                );
                for stmt in prog {
                    stmt.emit(&mut c).unwrap();
                }
                c.cpushop(VmOpcode::OP_JMP_LONG);
                c.cpush32(importer_ip);
                self.interned_strings = c.interned_strings.take().unwrap();
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
