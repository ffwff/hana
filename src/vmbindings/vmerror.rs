//! Provides an interface for virtual machine errors

use super::value::Value;
use super::vm::Vm;

#[repr(C)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types, dead_code)]
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
    ERROR_OP_BITWISE_AND,
    ERROR_OP_BITWISE_OR,
    ERROR_OP_BITWISE_XOR,
    ERROR_UNDEFINED_GLOBAL_VAR,
    ERROR_RECORD_NO_CONSTRUCTOR,
    ERROR_CONSTRUCTOR_NOT_FUNCTION,
    ERROR_MISMATCH_ARGUMENTS,
    ERROR_EXPECTED_CALLABLE,
    ERROR_CANNOT_ACCESS_NON_RECORD,
    ERROR_KEY_NON_INT,
    ERROR_RECORD_KEY_NON_STRING,
    ERROR_UNBOUNDED_ACCESS,
    ERROR_EXPECTED_RECORD_ARRAY,
    ERROR_CASE_EXPECTS_DICT,
    ERROR_UNHANDLED_EXCEPTION,
    ERROR_EXPECTED_ITERABLE,
    ERROR_EXPECTED_RECORD_OF_EXPR,
    ERROR_UNKNOWN_KEY,
}

#[cfg_attr(tarpaulin, skip)]
impl VmError {
    fn method_for_op(&self) -> &str {
        match self {
            VmError::ERROR_OP_ADD => "addition",
            VmError::ERROR_OP_SUB => "subtraction",
            VmError::ERROR_OP_MUL => "multiplication",
            VmError::ERROR_OP_DIV => "division",
            VmError::ERROR_OP_MOD => "modulo",
            VmError::ERROR_OP_AND => "logical and",
            VmError::ERROR_OP_OR => "logical or",
            VmError::ERROR_OP_LT => "less than",
            VmError::ERROR_OP_LEQ => "less than or equal to",
            VmError::ERROR_OP_GT => "greater than",
            VmError::ERROR_OP_GEQ => "greater than or equal to",
            VmError::ERROR_OP_EQ => "equality",
            VmError::ERROR_OP_NEQ => "inequality",
            _ => unreachable!(),
        }
    }

    pub unsafe fn hint(&self, vm: &Vm) -> Option<String> {
        match self {
            VmError::ERROR_OP_ADD
            | VmError::ERROR_OP_SUB
            | VmError::ERROR_OP_MUL
            | VmError::ERROR_OP_DIV
            | VmError::ERROR_OP_MOD
            | VmError::ERROR_OP_AND
            | VmError::ERROR_OP_OR
            | VmError::ERROR_OP_LT
            | VmError::ERROR_OP_LEQ
            | VmError::ERROR_OP_GT
            | VmError::ERROR_OP_GEQ
            | VmError::ERROR_OP_EQ
            | VmError::ERROR_OP_NEQ => {
                let left = vm.stack[vm.stack.len() - 2].unwrap();
                let right = vm.stack[vm.stack.len() - 1].unwrap();
                Some(format!(
                    "Can't perform {} between {} and {}",
                    self.method_for_op(),
                    left.type_name(),
                    right.type_name()
                ))
            }
            VmError::ERROR_MISMATCH_ARGUMENTS => Some(format!(
                "Function expects exactly {} arguments",
                vm.error_expected
            )),
            VmError::ERROR_UNBOUNDED_ACCESS => {
                Some(format!("Index must be between [0, {})", vm.error_expected))
            }
            VmError::ERROR_UNHANDLED_EXCEPTION => {
                let top = vm.stack.last().unwrap().unwrap();
                Some(match top {
                    Value::Record(rec) => {
                        let rec = rec.as_ref();
                        let mut lines = Vec::new();
                        if let Some(what) = rec.get("what") {
                            lines.push(format!("  what => {:?}", what.unwrap()));
                        }
                        if let Some(why) = rec.get("why") {
                            lines.push(format!("  why => {:?}", why.unwrap()));
                        }
                        if let Some(where_) = rec.get("where") {
                            lines.push(format!("  where => {:?}", where_.unwrap()));
                        }
                        if lines.is_empty() {
                            "The exception was a record".to_string()
                        } else {
                            format!(
                                "The exception gave the following hints:\n{}",
                                lines.join("\n")
                            )
                        }
                    }
                    _ => format!("The exception was {:?}", top),
                })
            }
            _ => None,
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
impl std::fmt::Display for VmError {
    #[allow(non_snake_case)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VmError::ERROR_OP_ADD
            | VmError::ERROR_OP_SUB
            | VmError::ERROR_OP_MUL
            | VmError::ERROR_OP_DIV
            | VmError::ERROR_OP_MOD
            | VmError::ERROR_OP_AND
            | VmError::ERROR_OP_OR
            | VmError::ERROR_OP_LT
            | VmError::ERROR_OP_LEQ
            | VmError::ERROR_OP_GT
            | VmError::ERROR_OP_GEQ
            | VmError::ERROR_OP_EQ
            | VmError::ERROR_OP_NEQ => write!(f, "Invalid arguments for {}", self.method_for_op()),
            VmError::ERROR_UNDEFINED_GLOBAL_VAR => write!(f, "Global variable is not defined"),
            VmError::ERROR_RECORD_NO_CONSTRUCTOR => {
                write!(f, "Cannot call record that has no constructor")
            }
            VmError::ERROR_CONSTRUCTOR_NOT_FUNCTION => {
                write!(f, "Cannot call record with non-function constructor")
            }
            VmError::ERROR_MISMATCH_ARGUMENTS => write!(f, "Argument mismatch"),
            VmError::ERROR_EXPECTED_CALLABLE => {
                write!(f, "Expected value to be callable (record or function)")
            }
            VmError::ERROR_CANNOT_ACCESS_NON_RECORD => {
                write!(f, "Cannot access property of a nil literal")
            }
            VmError::ERROR_KEY_NON_INT => write!(f, "Index must be an integer value"),
            VmError::ERROR_RECORD_KEY_NON_STRING => {
                write!(f, "Record key must be an string value")
            }
            VmError::ERROR_UNBOUNDED_ACCESS => write!(
                f,
                "Accessing an index that lies outside of the object's bound"
            ),
            VmError::ERROR_EXPECTED_RECORD_ARRAY => {
                write!(f, "Expected record or array to set an index for")
            }
            VmError::ERROR_CASE_EXPECTS_DICT => {
                write!(f, "case statement expects a dictionary as handler type")
            }
            VmError::ERROR_UNHANDLED_EXCEPTION => write!(f, "Unhandled exception"),
            VmError::ERROR_EXPECTED_ITERABLE => write!(f, "Expected iterable record or array"),
            VmError::ERROR_UNKNOWN_KEY => write!(f, "Unknown key"),
            _ => write!(f, "[vmerror]"),
        }
    }
}
