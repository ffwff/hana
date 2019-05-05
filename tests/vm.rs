extern crate haru;

#[cfg(test)]
pub mod vm_tests {

    use haru::vm::Vm;
    use haru::vm::VmOpcode;
    use haru::vm::Value;

    //#region numbers
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
        vm.code.push(VmOpcode::OP_PUSHF32);
        vm.cpushf32(1.5);
        vm.code.push(VmOpcode::OP_PUSHF32);
        vm.cpushf32(1.5);
        vm.code.push(VmOpcode::OP_ADD);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Float(3.0));
    }
    // #endregion

    // #region string
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
        assert_eq!(vm.stack.top().unwrap(), Value::Str("TestTest"));
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
        assert_eq!(vm.stack.top().unwrap(), Value::Str("TestTest"));
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
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.top().unwrap(), Value::Int(42));
    }
    // #endregion

}