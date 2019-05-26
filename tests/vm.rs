extern crate haru;

#[cfg(test)]
pub mod vm_tests {

    use haru::vmbindings::value::Value;
    use haru::vmbindings::vm::{Vm, VmOpcode};

    //#region numbers
    #[test]
    fn push_16() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH16);
        vm.cpush16(40000);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(40000));
    }

    #[test]
    fn push_32() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH32);
        vm.cpush32(100000);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(100000));
    }

    #[test]
    fn push_float() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(0.645);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(0.645));
    }

    #[test]
    fn add_ints() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(10);
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(11);
        vm.code.push(VmOpcode::OP_ADD);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(21));
    }

    #[test]
    fn add_floats() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_ADD);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(3.0));
    }

    #[test]
    fn div_floats() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.1);
        vm.code.push(VmOpcode::OP_DIV);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(1.5 / 1.1));
    }

    #[test]
    fn div_float_and_int() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_PUSH64);
        vm.cpush64(15);
        vm.code.push(VmOpcode::OP_DIV);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(1.5 / 15.0));
    }
    // #endregion

    // #region string
    #[test]
    fn string_basic() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(
            *vm.stack.top().unwrap().string(),
            String::from("Test")
        );
    }

    #[test]
    fn string_append() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_ADD);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(
            *vm.stack.top().unwrap().string(),
            String::from("TestTest")
        );
    }

    #[test]
    fn string_repeat() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(2);
        vm.code.push(VmOpcode::OP_MUL);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(
            *vm.stack.top().unwrap().string(),
            String::from("TestTest")
        );
    }
    // #endregion

    // #region vars
    #[test]
    fn global_var() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(42);
        vm.code.push(VmOpcode::OP_SET_GLOBAL);
        vm.cpushs("abc");
        vm.code.push(VmOpcode::OP_POP);
        vm.code.push(VmOpcode::OP_GET_GLOBAL);
        vm.cpushs("abc");
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(42));
    }
    // #endregion

    // #region unary ops
    #[test]
    fn op_not() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(42);
        vm.code.push(VmOpcode::OP_NOT);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(0));
    }

    #[test]
    fn negate_int() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(1);
        vm.code.push(VmOpcode::OP_NEGATE);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(-1));
    }

    #[test]
    fn negate_float() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_NEGATE);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(-1.5));
    }
    // #endregion

}
