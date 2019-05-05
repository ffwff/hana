use crate::vm::Vm;
use crate::vm::VmOpcode;

// private
struct Scope {
    vars : Vec<String>
}
impl Scope {
    fn new() -> Scope {
        Scope { vars: Vec::with_capacity(std::u8::MAX as usize) }
    }
}

// public
pub struct Compiler {
    scopes : Vec<Scope>,
    pub vm : Vm
}
impl Compiler {
    pub fn new() -> Compiler {
        Compiler{
            scopes: Vec::new(),
            vm: Vm::new()
        }
    }

    // local
    fn get_local(&self, var : String) -> Option<(u8, u8)> {
        let mut relascope : u8 = 0;
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.vars.iter().position(|x| *x == var) {
                return Some((slot as u8, relascope));
            }
            relascope += 1;
        }
        None
    }

    fn set_local(&mut self, var : String) -> Option<(u8, u8)> {
        if let Some(last) = self.scopes.last_mut() {
            last.vars.push(var);
            let idx = self.scopes.len()-1;
            return Some((idx as u8, 0));
        }
        None
    }

    // emit set var
    pub fn emit_set_var(&mut self, var : String) {
        // TODO: set locals
        self.vm.code.push(VmOpcode::OP_SET_GLOBAL);
        self.vm.cpushs(var);
    }
}
