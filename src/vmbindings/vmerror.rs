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
    ERROR_UNDEFINED_GLOBAL_VAR,
    ERROR_RECORD_NO_CONSTRUCTOR,
    ERROR_CONSTRUCTOR_NOT_FUNCTION,
    ERROR_MISMATCH_ARGUMENTS,
    ERROR_EXPECTED_CALLABLE,
    ERROR_CANNOT_ACCESS_NIL,
    ERROR_CANNOT_ACCESS_NON_RECORD,
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
        VmError::ERROR_CANNOT_ACCESS_NON_RECORD => write!(f, "Cannot access a literal that has no \"prototype\" key"),
        VmError::ERROR_KEY_NON_INT              => write!(f, "Index must be an integer value"),
        VmError::ERROR_RECORD_KEY_NON_STRING    => write!(f, "Record key must be an string value"),
        VmError::ERROR_UNBOUNDED_ACCESS         => write!(f, "Accessing a value that lies outside of the object's bound"),
        VmError::ERROR_EXPECTED_RECORD_ARRAY    => write!(f, "Expected record or array to set an index for"),
        VmError::ERROR_CASE_EXPECTS_DICT        => write!(f, "case statement expects a dictionary as handler type"),
        VmError::ERROR_UNHANDLED_EXCEPTION      => write!(f, "Unhandled exception"),
        _ => write!(f, "[vmerror]")
        }
    }

}