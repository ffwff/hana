//! Provides an abstraction for native values

use super::function::Function;
use super::gc::Gc;
use super::record::Record;
use super::string::HaruString;
use super::vm::Vm;
use super::vmerror::VmError;
use std::borrow::Borrow;
extern crate libc;

pub type NativeFnData = extern "C" fn(*mut Vm, u16);

#[derive(Clone, PartialEq)]
#[allow(non_camel_case_types, dead_code)]
/// Wrapper for native values
pub enum Value {
    // we don't have control over how rust manages its variant
    // types, so this is a convenient wrapper for (de)serialising
    // hana's values
    Nil,
    Int(i64),
    Float(f64),
    NativeFn(NativeFnData),
    Fn(Gc<Function>),
    Str(Gc<HaruString>),
    Record(Gc<Record>),
    Array(Gc<Vec<Value>>),

    PropagateError,
}

impl Value {
    // prototype
    pub fn get_prototype(&self, vm: *const Vm) -> *const Record {
        unimplemented!()
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn type_name(&self) -> &str {
        match self {
            Value::Nil => "nil",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::NativeFn(_) | Value::Fn(_) => "Function",
            Value::Str(_) => "String",
            Value::Record(_) => "Record",
            Value::Array(_) => "Array",
            _ => "unk",
        }
    }

    // bool
    pub fn is_true(self) -> bool {
        // TODO document this
        match self {
            Value::Int(x) => x > 0,
            Value::Float(x) => x > 0.0,
            Value::Str(x) => !x.as_ref().is_empty(),
            _ => false
        }
    }

    // #region binary ops

    // #region arithmetic
    pub fn add(self, other: Value, vm: &Vm) -> Result<Value, VmError> {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + y as f64)),
            (Value::Str(x), Value::Str(y)) => {
                let mut string = x.as_ref().to_string();
                string += y.as_ref();
                Ok(Value::Str(vm.malloc(string.into())))
            },
            _ => Err(VmError::ERROR_OP_ADD),
        }
    }
    pub fn sub(self, other: Value, vm: &Vm) -> Result<Value, VmError> {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 - y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - y as f64)),
            _ => Err(VmError::ERROR_OP_SUB),
        }
    }
    pub fn mul(self, other: Value, vm: &Vm) -> Result<Value, VmError> {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * y as f64)),
            (Value::Str(x), Value::Int(y)) => Ok(Value::Str(vm.malloc(x.as_ref().repeat(y as usize).into()))),
            _ => Err(VmError::ERROR_OP_MUL),
        }
    }
    pub fn div(self, other: Value, vm: &Vm) -> Result<Value, VmError> {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Float(x as f64 / y as f64)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 / y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x / y as f64)),
            _ => Err(VmError::ERROR_OP_DIV),
        }
    }
    pub fn r#mod(self, other: Value, vm: &Vm) -> Result<Value, VmError> {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x % y)),
            _ => Err(VmError::ERROR_OP_MOD),
        }
    }
    // #region in place
    pub fn add_in_place(self, other: Value, vm: &Vm) -> Result<(bool, Value), VmError> {
        match (&self, &other) {
            (Value::Str(x), Value::Str(y)) => {
                use std::borrow::BorrowMut;
                let string: &mut String = x.as_mut().borrow_mut();
                string.push_str(y.as_ref());
                Ok((true, self))
            },
            _ => self.add(other, vm).map(|val| (false, val)),
        }
    }
    pub fn mul_in_place(self, other: Value, vm: &Vm) -> Result<(bool, Value), VmError> {
        match (&self, &other) {
            (Value::Str(x), Value::Int(y)) => {
                use std::borrow::BorrowMut;
                let string: &mut String = x.as_mut().borrow_mut();
                string.push_str(&string.clone().as_str().repeat(*y as usize - 1));
                Ok((true, self))
            },
            _ => self.mul(other, vm).map(|val| (false, val)),
        }
    }
    // #endregion
    // #endregion

    // #region comparison
    pub fn lt(self, other: Value) -> bool {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => x < y,
            (Value::Float(x), Value::Float(y)) => x < y,
            (Value::Int(x), Value::Float(y)) => (x as f64) < y,
            (Value::Float(x), Value::Int(y)) => x < (y as f64),
            (Value::Str(x), Value::Str(y)) => x.as_ref().as_str() < y.as_ref().as_str(),
            _ => false,
        }
    }
    pub fn gt(self, other: Value) -> bool {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => x > y,
            (Value::Float(x), Value::Float(y)) => x > y,
            (Value::Int(x), Value::Float(y)) => (x as f64) > y,
            (Value::Float(x), Value::Int(y)) => x > (y as f64),
            (Value::Str(x), Value::Str(y)) => x.as_ref().as_str() > y.as_ref().as_str(),
            _ => false,
        }
    }
    pub fn eq(self, other: Value) -> bool {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Int(x), Value::Float(y)) => (x as f64) == y,
            (Value::Float(x), Value::Int(y)) => x == (y as f64),
            (Value::Str(x), Value::Str(y)) => x.as_ref().as_str() == y.as_ref().as_str(),
            _ => false,// TODO
            //(x, y) => x.wrap() == y.wrap(),
        }
    }
    pub fn geq(self, other: Value) -> bool {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => x >= y,
            (Value::Float(x), Value::Float(y)) => x >= y,
            (Value::Int(x), Value::Float(y)) => (x as f64) >= y,
            (Value::Float(x), Value::Int(y)) => x >= (y as f64),
            (Value::Str(x), Value::Str(y)) => x.as_ref().as_str() >= y.as_ref().as_str(),
            _ => false,
        }
    }
    pub fn leq(self, other: Value) -> bool {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => x <= y,
            (Value::Float(x), Value::Float(y)) => x <= y,
            (Value::Int(x), Value::Float(y)) => (x as f64) <= y,
            (Value::Float(x), Value::Int(y)) => x <= (y as f64),
            (Value::Str(x), Value::Str(y)) => x.as_ref().as_str() <= y.as_ref().as_str(),
            _ => false,
        }
    }
    pub fn neq(self, other: Value) -> bool {
        !self.eq(other)
    }
    // #endregion

    // #endregion

    // #region unary ops
    pub fn negate(self, vm: &Vm) -> Result<Value, VmError> {
        match self {
            Value::Int(x) => Ok(Value::Int(-x)),
            Value::Float(x) => Ok(Value::Float(-x)),
            other => Ok(other),
        }
    }
    // #endregion

    pub fn as_gc_pointer(&self) -> Option<*mut libc::c_void> {
        unimplemented!()
    }

    pub fn ref_inc(&self) {
        return;
    }
    pub fn ref_dec(&self) {
        return;
    }

}

use std::fmt;

#[cfg_attr(tarpaulin, skip)]
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "[nil]"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::NativeFn(_) => write!(f, "[native fn]"),
            Value::Fn(_) => write!(f, "[fn]"),
            Value::Str(p) => write!(f, "{}", p.as_ref().borrow() as &String),
            Value::Record(p) => write!(f, "[record {:p}]", p.to_raw()),
            Value::Array(p) => write!(f, "[array {:p}]", p.to_raw()),
            _ => unreachable!(),
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "[nil]"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::NativeFn(_) => write!(f, "[native fn]"),
            Value::Fn(_) => write!(f, "[fn]"),
            Value::Str(p) => {
                let mut s = String::new();
                let p = p.as_ref().borrow();
                for ch in (p as &String).chars() {
                    match ch {
                        '\n' => s.push_str("\\n"),
                        '"' => s.push('"'),
                        _ => s.push(ch),
                    }
                }
                write!(f, "\"{}\"", s)
            }
            Value::Record(p) => write!(f, "[record {:p}]", p.to_raw()),
            Value::Array(p) => write!(f, "[array {:p}]", p.to_raw()),
            _ => write!(f, "[unk]"),
        }
    }
}
