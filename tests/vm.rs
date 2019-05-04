extern crate haru;

#[cfg(test)]
pub mod vm_tests {

    use haru::vm::Vm;
    use haru::vm::VmOpcode;
    use haru::vm::Value;

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


}