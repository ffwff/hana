mod value_ext;
use value_ext::*;
extern crate haru;
use haru::compiler::Compiler;
use haru::vmbindings::value::Value;
use haru::vmbindings::vm::VmOpcode;

#[cfg(test)]
pub mod vm_tests {

    use super::*;

    //#region numbers
    #[test]
    fn push_16() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSH16);
        c.cpush16(40000);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Int(40000));
    }

    #[test]
    fn push_32() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSH32);
        c.cpush32(100000);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Int(100000));
    }

    #[test]
    fn push_float() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(0.645);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Float(0.645));
    }

    #[test]
    fn add_ints() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSH8);
        c.cpush8(10);
        c.cpushop(VmOpcode::OP_PUSH8);
        c.cpush8(11);
        c.cpushop(VmOpcode::OP_ADD);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Int(21));
    }

    #[test]
    fn add_floats() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(1.5);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(1.5);
        c.cpushop(VmOpcode::OP_ADD);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Float(3.0));
    }

    #[test]
    fn div_floats() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(1.5);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(1.1);
        c.cpushop(VmOpcode::OP_DIV);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Float(1.5 / 1.1));
    }

    #[test]
    fn div_float_and_int() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(1.5);
        c.cpushop(VmOpcode::OP_PUSH64);
        c.cpush64(15);
        c.cpushop(VmOpcode::OP_DIV);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Float(1.5 / 15.0));
    }
    // #endregion

    // #region string
    #[test]
    fn string_basic() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHSTR);
        c.cpushs("Test");
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(
            *vm.stack.last().unwrap().unwraps().string(),
            String::from("Test")
        );
    }

    #[test]
    fn string_append() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHSTR);
        c.cpushs("Test");
        c.cpushop(VmOpcode::OP_PUSHSTR);
        c.cpushs("Test");
        c.cpushop(VmOpcode::OP_ADD);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(
            *vm.stack.last().unwrap().unwraps().string(),
            String::from("TestTest")
        );
    }

    #[test]
    fn string_repeat() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHSTR);
        c.cpushs("Test");
        c.cpushop(VmOpcode::OP_PUSH8);
        c.cpush8(2);
        c.cpushop(VmOpcode::OP_MUL);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(
            *vm.stack.last().unwrap().unwraps().string(),
            String::from("TestTest")
        );
    }
    // #endregion

    // #region vars
    #[test]
    fn global_var() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSH8);
        c.cpush8(42);
        c.cpushop(VmOpcode::OP_SET_GLOBAL);
        c.cpushs("abc");
        c.cpushop(VmOpcode::OP_POP);
        c.cpushop(VmOpcode::OP_GET_GLOBAL);
        c.cpushs("abc");
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Int(42));
    }
    // #endregion

    // #region unary ops
    #[test]
    fn op_not() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSH8);
        c.cpush8(42);
        c.cpushop(VmOpcode::OP_NOT);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Int(0));
    }

    #[test]
    fn negate_int() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSH8);
        c.cpush8(1);
        c.cpushop(VmOpcode::OP_NEGATE);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Int(-1));
    }

    #[test]
    fn negate_float() {
        let mut c = Compiler::new(false);
        c.cpushop(VmOpcode::OP_PUSHF64);
        c.cpushf64(1.5);
        c.cpushop(VmOpcode::OP_NEGATE);
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.last().unwrap().unwraps(), Value::Float(-1.5));
    }
    // #endregion

}
