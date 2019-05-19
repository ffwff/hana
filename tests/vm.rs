extern crate haru;

#[cfg(test)]
pub mod vm_tests {

    use haru::vm::Vm;
    use haru::vm::VmOpcode;
    use haru::vm::Value;
    use haru::gc;

    //#region numbers
    #[test]
    fn push_16() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH16);
        vm.cpush16(40000);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(40000));
    }

    #[test]
    fn push_32() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH32);
        vm.cpush32(100000);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(100000));
    }

    #[test]
    fn push_float() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(0.645);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(0.645));
    }

    #[test]
    fn add_ints() {
        gc::disable();
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
        gc::disable();
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
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.1);
        vm.code.push(VmOpcode::OP_DIV);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(1.5/1.1));
    }

    #[test]
    fn div_float_and_int() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHF64);
        vm.cpushf64(1.5);
        vm.code.push(VmOpcode::OP_PUSH64);
        vm.cpush64(15);
        vm.code.push(VmOpcode::OP_DIV);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(1.5/15.0));
    }
    // #endregion

    // #region string
    #[test]
    fn string_basic() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(*vm.stack.top().unwrap().string(), String::from("Test"));
    }

    #[test]
    fn string_append() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_ADD);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(*vm.stack.top().unwrap().string(), String::from("TestTest"));
    }

    #[test]
    fn string_repeat() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSHSTR);
        vm.cpushs("Test");
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(2);
        vm.code.push(VmOpcode::OP_MUL);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(*vm.stack.top().unwrap().string(), String::from("TestTest"));
    }
    // #endregion

    // #region vars
    #[test]
    fn global_var() {
        gc::disable();
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

    // #region vars
    #[test]
    fn op_not() {
        gc::disable();
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.cpush8(42);
        vm.code.push(VmOpcode::OP_NOT);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(0));
    }
    // #endregion
}