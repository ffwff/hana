extern crate haru;

#[cfg(test)]
pub mod vm_tests {

    use haru::vm::Vm;
    use haru::vm::VmOpcode;

    #[test]
    fn add_ints() {
        let mut vm = Vm::new();
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.code.push(VmOpcode::OP_PUSH8);
        vm.code.push(VmOpcode::OP_ADD);
        vm.code.push(VmOpcode::OP_HALT);
        vm.execute();
        assert_eq!(vm.stack.len(), 1);
    }


}