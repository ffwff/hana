extern crate haru;

#[cfg(test)]
pub mod vm_tests {

    use haru::vmbindings::vm::{Vm, VmOpcode};
    use haru::vmbindings::value::Value;


    //#region numbers
    #[test]
    fn push_16() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH16);
        vm.borrow_mut().cpush16(40000);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Int(40000));
    }

    #[test]
    fn push_32() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH32);
        vm.borrow_mut().cpush32(100000);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Int(100000));
    }

    #[test]
    fn push_float() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(0.645);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Float(0.645));
    }

    #[test]
    fn add_ints() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH8);
        vm.borrow_mut().cpush8(10);
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH8);
        vm.borrow_mut().cpush8(11);
        vm.borrow_mut().code.push(VmOpcode::OP_ADD);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Int(21));
    }

    #[test]
    fn add_floats() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(1.5);
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(1.5);
        vm.borrow_mut().code.push(VmOpcode::OP_ADD);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Float(3.0));
    }

    #[test]
    fn div_floats() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(1.5);
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(1.1);
        vm.borrow_mut().code.push(VmOpcode::OP_DIV);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Float(1.5/1.1));
    }

    #[test]
    fn div_float_and_int() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(1.5);
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH64);
        vm.borrow_mut().cpush64(15);
        vm.borrow_mut().code.push(VmOpcode::OP_DIV);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Float(1.5/15.0));
    }
    // #endregion

    // #region string
    #[test]
    fn string_basic() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHSTR);
        vm.borrow_mut().cpushs("Test");
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(*vm.borrow().stack.top().unwrap().string(), String::from("Test"));
    }

    #[test]
    fn string_append() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHSTR);
        vm.borrow_mut().cpushs("Test");
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHSTR);
        vm.borrow_mut().cpushs("Test");
        vm.borrow_mut().code.push(VmOpcode::OP_ADD);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(*vm.borrow().stack.top().unwrap().string(), String::from("TestTest"));
    }

    #[test]
    fn string_repeat() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHSTR);
        vm.borrow_mut().cpushs("Test");
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH8);
        vm.borrow_mut().cpush8(2);
        vm.borrow_mut().code.push(VmOpcode::OP_MUL);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(*vm.borrow().stack.top().unwrap().string(), String::from("TestTest"));
    }
    // #endregion

    // #region vars
    #[test]
    fn global_var() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH8);
        vm.borrow_mut().cpush8(42);
        vm.borrow_mut().code.push(VmOpcode::OP_SET_GLOBAL);
        vm.borrow_mut().cpushs("abc");
        vm.borrow_mut().code.push(VmOpcode::OP_POP);
        vm.borrow_mut().code.push(VmOpcode::OP_GET_GLOBAL);
        vm.borrow_mut().cpushs("abc");
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Int(42));
    }
    // #endregion

    // #region unary ops
    #[test]
    fn op_not() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH8);
        vm.borrow_mut().cpush8(42);
        vm.borrow_mut().code.push(VmOpcode::OP_NOT);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Int(0));
    }

    #[test]
    fn negate_int() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSH8);
        vm.borrow_mut().cpush8(1);
        vm.borrow_mut().code.push(VmOpcode::OP_NEGATE);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Int(-1));
    }

    #[test]
    fn negate_float() {
        let vm = Vm::new();
        vm.borrow_mut().code.push(VmOpcode::OP_PUSHF64);
        vm.borrow_mut().cpushf64(1.5);
        vm.borrow_mut().code.push(VmOpcode::OP_NEGATE);
        vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        vm.borrow().execute();
        assert_eq!(vm.borrow().stack.len(), 1);
        assert_eq!(vm.borrow().stack.top().unwrap(), Value::Float(-1.5));
    }
    // #endregion

}