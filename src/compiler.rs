use crate::vm::Vm;
use crate::vm::VmOpcode;

// private
struct Scope {
    vars : Vec<String>
}
impl Scope {
    fn new() -> Scope {
        Scope { vars: Vec::with_capacity(std::u16::MAX as usize) }
    }
}

//
struct LoopStatement {
    pub fill_continue: Vec<usize>,
    pub fill_break: Vec<usize>
}

// public
pub struct Compiler {
    scopes : Vec<Scope>,
    loop_stmts : Vec<LoopStatement>,
    pub vm : Vm
}
impl Compiler {
    pub fn new() -> Compiler {
        Compiler{
            scopes: Vec::new(),
            loop_stmts: Vec::new(),
            vm: Vm::new()
        }
    }

    // local
    fn get_local(&self, var : &String) -> Option<(u16, u16)> {
        let mut relascope : u16 = 0;
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.vars.iter().position(|x| *x == *var) {
                return Some((slot as u16, relascope));
            }
            relascope += 1;
        }
        None
    }

    pub fn set_local(&mut self, var : String) -> Option<(u16, u16)> {
        if let Some(last) = self.scopes.last_mut() {
            last.vars.push(var);
            let idx = self.scopes.len()-1;
            return Some((idx as u16, 0));
        }
        None
    }

    // emit set var
    pub fn emit_set_var(&mut self, var : String) {
        if var.starts_with("$") || self.scopes.len() == 0 {
            // set global
            self.vm.code.push(VmOpcode::OP_SET_GLOBAL);
            self.vm.cpushs(if var.starts_with("$") { &var[1..] }
                           else { var.as_str() });
        } else if let Some(local) = self.get_local(&var) {
            // set existing local
            let mut slot = local.0;
            let relascope = local.1;
            if relascope != 0 {
                let local = self.set_local(var.clone()).unwrap();
                slot = local.0;
            }
            self.vm.code.push(VmOpcode::OP_SET_LOCAL);
            self.vm.cpush16(slot);
        } else {
            let local = self.set_local(var.clone()).unwrap();
            let slot = local.0;
            self.vm.code.push(VmOpcode::OP_SET_LOCAL);
            self.vm.cpush16(slot);
        }
    }
    pub fn emit_set_var_fn(&mut self, var : String) {
        if var.starts_with("$") || self.scopes.len() == 0 {
            // set global
            self.vm.code.push(VmOpcode::OP_SET_GLOBAL);
            self.vm.cpushs(if var.starts_with("$") { &var[1..] }
                           else { var.as_str() });
        } else if let Some(local) = self.get_local(&var) {
            // set existing local
            let mut slot = local.0;
            let relascope = local.1;
            if relascope != 0 {
                let local = self.set_local(var.clone()).unwrap();
                slot = local.0;
            }
            self.vm.code.push(VmOpcode::OP_SET_LOCAL_FUNCTION_DEF);
            self.vm.cpush16(slot);
        } else {
            let local = self.set_local(var.clone()).unwrap();
            let slot = local.0;
            self.vm.code.push(VmOpcode::OP_SET_LOCAL_FUNCTION_DEF);
            self.vm.cpush16(slot);
        }
    }

    pub fn emit_get_var(&mut self, var : String) {
        let local = self.get_local(&var);
        if var.starts_with("$") || !local.is_some() {
            // set global
            self.vm.code.push(VmOpcode::OP_GET_GLOBAL);
            self.vm.cpushs(if var.starts_with("$") { &var[1..] }
                           else { var.as_str() });
        } else {
            let local = local.unwrap();
            let slot = local.0;
            let relascope = local.1;
            if relascope == 0 {
                self.vm.code.push(VmOpcode::OP_GET_LOCAL);
                self.vm.cpush16(slot);
            } else {
                self.vm.code.push(VmOpcode::OP_GET_LOCAL_UP);
                self.vm.cpush16(slot);
                self.vm.cpush16(relascope);
            }
        }
    }

    // labels
    pub fn reserve_label(&mut self) -> usize {
        let pos = self.vm.code.len();
        self.vm.cpush32(0xdeadbeef);
        pos
    }

    pub fn fill_label(&mut self, pos: usize, label: usize) {
        self.vm.cfill_label(pos, label);
    }

    pub fn reserve_label16(&mut self) -> usize {
        let pos = self.vm.code.len();
        self.vm.cpush16(0);
        pos
    }
    pub fn fill_label16(&mut self, pos: usize, label: u16) {
        self.vm.cfill_label16(pos, label);
    }

    // scopes
    pub fn scope(&mut self) {
        self.scopes.push(Scope::new());
    }
    pub fn unscope(&mut self) -> u16 {
        let size = self.scopes.pop().unwrap().vars.len();
        size as u16
    }

    // loops
    pub fn loop_start(&mut self) {
        self.loop_stmts.push(LoopStatement{
            fill_continue: Vec::new(),
            fill_break: Vec::new(),
        });
    }
    pub fn loop_continue(&mut self) {
        let label = self.reserve_label();
        let ls = self.loop_stmts.last_mut().unwrap();
        ls.fill_continue.push(label);
    }
    pub fn loop_break(&mut self) {
        let label = self.reserve_label();
        let ls = self.loop_stmts.last_mut().unwrap();
        ls.fill_break.push(label);
    }
    pub fn loop_end(&mut self, next_it_pos : usize, end_pos : usize) {
        let ls = self.loop_stmts.pop().unwrap();
        for label in ls.fill_continue {
            self.fill_label(label, next_it_pos);
        }
        for label in ls.fill_break {
            self.fill_label(label, end_pos);
        }
    }
}
